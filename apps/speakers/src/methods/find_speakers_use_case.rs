use serde_json::{json, Value};
use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{Cancel, Failure};
use loomasplugins_sound_reader::read_audio_region;
use loomasplugins_work_files::{unique_work_file_name, write_wav_file};

use crate::cluster::{build_turns, cluster_voices};
use crate::models::{device_report, locate_models};
use crate::segment::{segment_windows, Window, RATE};
use crate::voiceprint::unit;
use crate::voices::{learn_pattern, match_voices, read_voice_patterns, VoicePattern};

use super::choose_sample_algorithm::choose_sample;
use super::print_pieces_use_case::{print_pieces, Origin};
use super::read_find_options_mapper::read_find_options;

fn rounded(value: f32) -> f64 {
    (f64::from(value) * 1000.0).round() / 1000.0
}

/// The voice of each frame of each local speaker of each window: the voice of the run the frame is
/// in, or, for a frame in a run too short for a print, the voice of the nearest run of the same
/// local speaker in that window (none when it has no run with a print).
fn frame_voices(windows: &[Window], origins: &[Origin], groups: &[usize]) -> Vec<[Vec<Option<usize>>; 3]> {
    windows
        .iter()
        .enumerate()
        .map(|(index, window)| {
            std::array::from_fn(|local| {
                let runs: Vec<(&Origin, usize)> = origins
                    .iter()
                    .zip(groups)
                    .filter(|(origin, _)| origin.window == index && origin.local == local)
                    .map(|(origin, group)| (origin, *group))
                    .collect();
                (0..window.frames.len())
                    .map(|frame| {
                        runs.iter()
                            .min_by_key(|(origin, _)| {
                                if frame < origin.first {
                                    origin.first - frame
                                } else {
                                    frame.saturating_sub(origin.last)
                                }
                            })
                            .map(|(_, group)| *group)
                    })
                    .collect()
            })
        })
        .collect()
}

/// The first `voice-N` id no pattern has, counting from after the highest one given.
fn new_ids(patterns: &[VoicePattern], count: usize) -> Vec<String> {
    let highest = patterns
        .iter()
        .filter_map(|pattern| pattern.id.strip_prefix("voice-")?.parse::<usize>().ok())
        .max()
        .unwrap_or(0);
    (1..=count).map(|n| format!("voice-{}", highest + n)).collect()
}

/// The `find` action: who speaks when in a video, and which voice each one is.
///
/// Params: `media` (the sound of a video, decoded by the host to 16 kHz, one channel), `voices`
/// (patterns from earlier runs, see `read_voice_patterns`) and the settings of
/// `read_find_options`. It gives `{ turns, voices, durationMs, device }`. A turn is
/// `{ label, voice, startMs, endMs, confidence }`: `label` is the name of a known voice or its id. A voice is `{ voice, name, known, similarity, learned, seconds, pattern, patternFile,
/// sample }`: `pattern` is the content of its `voice.json`, learned from this video when it
/// matched surely enough; `patternFile` and `sample` are that file and a WAV to listen to.
pub fn find_speakers(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let mut options = read_find_options(params)?;
    options.device = DeviceChoice::resolve(options.device, cancel.granted_device());
    let patterns = read_voice_patterns(params)?;
    let media = params
        .get("media")
        .ok_or_else(|| Failure::status(400, "media is missing: give a video"))?;
    let sound = read_audio_region(media)
        .map_err(|error| Failure::status(error.code, error.message))?
        .ok_or_else(|| {
            Failure::status(400, "media must be sound Loomas decoded for this plugin (it asks for 16 kHz sound)")
        })?;
    if sound.sample_rate != RATE {
        return Err(Failure::status(
            400,
            format!("this plugin listens at {RATE} Hz, not {} Hz", sound.sample_rate),
        ));
    }
    let work_dir = if options.files {
        Some(cancel.work_dir().ok_or_else(|| {
            Failure::status(422, "Loomas gave this plugin no folder to write the voice files in")
        })?)
    } else {
        None
    };
    let samples = &sound.samples;
    let duration_ms = sound.duration_ms();
    if samples.is_empty() {
        return Ok(json!({ "turns": [], "voices": [], "durationMs": 0, "device": null }));
    }

    let models = locate_models(cancel)?;
    let windows = segment_windows(&models.segmentation, options.device, samples, cancel)?;
    let printed = print_pieces(&models.voice_print, options.device, &windows, samples, cancel)?;
    let groups = cluster_voices(&printed.pieces, options.max_speakers);

    let local_voices = frame_voices(&windows, &printed.origins, &groups);
    let mut turns = build_turns(&windows, &local_voices, samples.len());

    // Number the voices that kept a turn in the order they first speak.
    let mut order: Vec<usize> = Vec::new();
    for turn in &turns {
        if !order.contains(&turn.voice) {
            order.push(turn.voice);
        }
    }
    for turn in &mut turns {
        turn.voice = order.iter().position(|voice| *voice == turn.voice).expect("every voice is ordered");
    }
    let prints: Vec<Vec<f32>> = order
        .iter()
        .map(|group| {
            let mut sum = vec![0.0f32; printed.pieces.first().map_or(0, |piece| piece.print.len())];
            for (piece, weight) in printed.pieces.iter().zip(&printed.seconds).zip(&groups).filter_map(
                |((piece, seconds), voice)| (voice == group).then_some((piece, seconds)),
            ) {
                for (total, value) in sum.iter_mut().zip(&piece.print) {
                    *total += value * weight;
                }
            }
            unit(sum)
        })
        .collect();
    let heard_seconds: Vec<f64> = (0..order.len())
        .map(|voice| {
            turns
                .iter()
                .filter(|turn| turn.voice == voice)
                .map(|turn| (turn.end_ms - turn.start_ms) as f64 / 1000.0)
                .sum()
        })
        .collect();

    let matched = match_voices(&prints, &patterns, options.min_similarity);
    let mut fresh = new_ids(&patterns, order.len()).into_iter();
    let mut described = Vec::with_capacity(order.len());
    let mut labels = Vec::with_capacity(order.len());
    for voice in 0..order.len() {
        let seconds = heard_seconds[voice];
        let (pattern, similarity, learned) = match matched[voice] {
            Some((index, similarity)) => {
                let known = &patterns[index];
                if options.learn && similarity >= options.learn_similarity {
                    (learn_pattern(known, &prints[voice], seconds), Some(similarity), true)
                } else {
                    (known.clone(), Some(similarity), false)
                }
            }
            None => (
                VoicePattern {
                    id: fresh.next().expect("one fresh id a voice"),
                    name: String::new(),
                    seconds,
                    print: prints[voice].clone(),
                },
                None,
                false,
            ),
        };
        labels.push((pattern.label().to_string(), pattern.id.clone()));
        let mut entry = json!({
            "voice": pattern.id,
            "name": if pattern.name.trim().is_empty() { Value::Null } else { json!(pattern.name.trim()) },
            "known": similarity.is_some(),
            "similarity": similarity.map(rounded),
            "learned": learned,
            "seconds": (seconds * 10.0).round() / 10.0,
            "pattern": pattern.to_value(),
        });
        if let Some(folder) = work_dir {
            let file = folder.join(unique_work_file_name("voice", "json"));
            std::fs::write(&file, serde_json::to_string_pretty(&entry["pattern"]).expect("JSON writes"))
                .map_err(|error| Failure::status(500, format!("{}: {error}", file.display())))?;
            entry["patternFile"] = json!({ "workFile": file.to_string_lossy(), "mime": "application/json" });
            if let Some((from, to)) = choose_sample(&turns, voice, options.sample_ms) {
                let at = |ms: u64| ((ms * u64::from(RATE) / 1000) as usize).min(samples.len());
                entry["sample"] = write_wav_file("voice-sample", &samples[at(from)..at(to)], RATE, folder)
                    .map_err(|error| Failure::status(500, error.to_string()))?;
            }
        }
        described.push(entry);
    }

    let turns: Vec<Value> = turns
        .iter()
        .map(|turn| {
            json!({
                "label": labels[turn.voice].0,
                "voice": labels[turn.voice].1,
                "startMs": sound.start_ms + turn.start_ms,
                "endMs": sound.start_ms + turn.end_ms,
                "confidence": rounded(turn.confidence),
            })
        })
        .collect();
    Ok(json!({
        "turns": turns,
        "voices": described,
        "durationMs": duration_ms,
        "device": device_report().map(|report| report.to_value()),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voiceprint::PRINT_SIZE;

    #[test]
    fn a_path_in_place_of_decoded_sound_and_a_missing_media_are_400s() {
        let failure = find_speakers(&json!({ "media": "a.mp4", "files": false }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        let failure = find_speakers(&json!({ "files": false }), &Cancel::default()).unwrap_err();
        assert!(failure.message.contains("media is missing"));
    }

    #[test]
    fn a_wrong_pattern_is_refused_before_anything_is_read_or_downloaded() {
        let failure = find_speakers(&json!({ "media": "a.mp4", "voices": [{ "kind": "nope" }] }), &Cancel::default())
            .unwrap_err();
        assert!(failure.message.contains("not a voice pattern"), "{}", failure.message);
    }

    #[test]
    fn new_voices_are_numbered_after_the_highest_known_id() {
        let known = |id: &str| VoicePattern { id: id.into(), name: String::new(), seconds: 1.0, print: vec![0.0; PRINT_SIZE] };
        assert_eq!(new_ids(&[], 2), ["voice-1", "voice-2"]);
        assert_eq!(new_ids(&[known("voice-3"), known("John")], 1), ["voice-4"]);
    }
}
