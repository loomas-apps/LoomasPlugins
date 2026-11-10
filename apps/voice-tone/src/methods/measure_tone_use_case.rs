use serde_json::{json, Map, Value};
use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{Cancel, Failure};
use loomasplugins_sound_reader::read_audio_region;

use crate::emotion::{classify_emotion, device_report, EMOTIONS};
use crate::loudness::{loudness_class, reference_level, speech_level};
use crate::models::locate_model;

use super::read_measure_options_mapper::read_measure_options;

/// The rate the model listens at, and the one the manifest asks Loomas to decode to.
const RATE: u64 = 16_000;

fn round(value: f64, places: i32) -> f64 {
    let scale = 10f64.powi(places);
    (value * scale).round() / scale
}

/// The windows over a sound from `start_ms` lasting `duration_ms`, each `window_ms` long (the
/// last one shorter), as items with `startMs` and `endMs`.
fn windows(start_ms: u64, duration_ms: u64, window_ms: u64) -> Vec<Value> {
    (0..duration_ms.div_ceil(window_ms))
        .map(|n| {
            let from = n * window_ms;
            json!({ "startMs": start_ms + from, "endMs": start_ms + (from + window_ms).min(duration_ms) })
        })
        .collect()
}

/// The `measure` action: how each stretch of speech is said.
///
/// Params: `media` (the sound of a video, decoded by the host to 16 kHz, one channel), and the
/// settings of `read_measure_options`. With `segments` (the sentences of speech-to-text or the
/// turns of speakers, as they are) each one is measured and comes back with its own fields. With
/// none the sound is measured in windows and the silent ones are left out. It gives
/// `{ segments, referenceDb, durationMs, device }`. Each segment gains `loudnessDb` (the level of
/// its speech, dB below full scale), `relativeDb` (against `referenceDb`, the middle level of all
/// the segments), `loudness` (`low`, `normal` or `loud`), and, unless `emotions` is false,
/// `emotion`, `emotionConfidence` and `emotions` (the probability of each). A value that cannot be
/// measured (silence, a stretch under half a second for the emotion, a time outside the sound)
/// is null.
pub fn measure_tone(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let mut options = read_measure_options(params)?;
    options.device = DeviceChoice::resolve(options.device, cancel.granted_device());
    let media = params
        .get("media")
        .ok_or_else(|| Failure::status(400, "media is missing: give a video"))?;
    let sound = read_audio_region(media)
        .map_err(|error| Failure::status(error.code, error.message))?
        .ok_or_else(|| {
            Failure::status(400, "media must be sound Loomas decoded for this plugin (it asks for 16 kHz sound)")
        })?;
    if u64::from(sound.sample_rate) != RATE {
        return Err(Failure::status(
            400,
            format!("this plugin listens at {RATE} Hz, not {} Hz", sound.sample_rate),
        ));
    }
    let duration_ms = sound.duration_ms();
    let given = options.segments.is_some();
    let items = options
        .segments
        .take()
        .unwrap_or_else(|| windows(sound.start_ms, duration_ms, options.window_ms));
    let samples_of = |item: &Value| -> &[f32] {
        let at = |key: &str| {
            let ms = item[key].as_u64().unwrap_or(0).saturating_sub(sound.start_ms);
            ((ms * RATE / 1000) as usize).min(sound.samples.len())
        };
        &sound.samples[at("startMs")..at("endMs")]
    };

    let levels: Vec<Option<f64>> = items.iter().map(|item| speech_level(samples_of(item))).collect();
    let (items, levels): (Vec<Value>, Vec<Option<f64>>) = items
        .into_iter()
        .zip(levels)
        .filter(|(_, level)| given || level.is_some())
        .unzip();
    let reference = reference_level(&levels.iter().flatten().copied().collect::<Vec<_>>());

    let model = if options.emotions && !items.is_empty() { Some(locate_model(cancel)?) } else { None };
    let mut measured = Vec::with_capacity(items.len());
    for (number, (item, level)) in items.iter().zip(&levels).enumerate() {
        if cancel.is_cancelled() {
            return Err(Failure::named("CANCELLED", "cancelled"));
        }
        cancel.progress(&format!("listening to stretch {} of {}", number + 1, items.len()));
        let mut fields: Map<String, Value> = item.as_object().cloned().unwrap_or_default();
        let relative = level.zip(reference).map(|(level, reference)| level - reference);
        fields.insert("loudnessDb".into(), json!(level.map(|level| round(level, 1))));
        fields.insert("relativeDb".into(), json!(relative.map(|relative| round(relative, 1))));
        fields.insert(
            "loudness".into(),
            json!(level.zip(reference).map(|(level, reference)| loudness_class(level, reference, options.loud_db))),
        );
        if let Some(file) = &model {
            let emotion = match level {
                Some(_) => classify_emotion(file, options.device, samples_of(item))?,
                None => None,
            };
            fields.insert("emotion".into(), json!(emotion.as_ref().map(|emotion| emotion.label)));
            fields.insert(
                "emotionConfidence".into(),
                json!(emotion.as_ref().map(|emotion| round(f64::from(emotion.confidence), 3))),
            );
            fields.insert(
                "emotions".into(),
                emotion.map_or(Value::Null, |emotion| {
                    EMOTIONS
                        .iter()
                        .zip(&emotion.scores)
                        .map(|(name, score)| ((*name).to_string(), json!(round(f64::from(*score), 3))))
                        .collect::<Map<String, Value>>()
                        .into()
                }),
            );
        }
        measured.push(Value::Object(fields));
    }
    Ok(json!({
        "segments": measured,
        "referenceDb": reference.map(|reference| round(reference, 1)),
        "durationMs": duration_ms,
        "device": device_report().map(|report| report.to_value()),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_windows_cover_the_sound_and_the_last_is_shorter() {
        let made = windows(1_000, 7_000, 3_000);
        let times: Vec<(u64, u64)> =
            made.iter().map(|w| (w["startMs"].as_u64().unwrap(), w["endMs"].as_u64().unwrap())).collect();
        assert_eq!(times, vec![(1_000, 4_000), (4_000, 7_000), (7_000, 8_000)]);
    }

    #[test]
    fn a_path_in_place_of_decoded_sound_and_a_missing_media_are_400s() {
        let failure = measure_tone(&serde_json::json!({ "media": "a.mp4" }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(serde_json::json!(400)));
        let failure = measure_tone(&serde_json::json!({}), &Cancel::default()).unwrap_err();
        assert!(failure.message.contains("media is missing"));
    }
}
