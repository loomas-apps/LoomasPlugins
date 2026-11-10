use serde_json::{json, Value};
use loomasplugins_frame_reader::{read_video_region, PixelFormat};
use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{Cancel, Failure};

use crate::detect::{detect_faces, Face};
use crate::expression::{classify_expression, EXPRESSIONS};
use crate::models::{device_report, locate_models};
use crate::track::{expression_runs, track_faces, Reading, Sighting};

use super::read_find_options_mapper::read_find_options;

/// A face not seen for longer than this is taken for another face when one shows again.
const TRACK_GAP_MS: u64 = 2_000;

fn round(value: f32) -> f64 {
    (f64::from(value) * 1000.0).round() / 1000.0
}

/// The `find` action: the faces in a video and the expression each shows over time.
///
/// Params: `media` (a video Loomas decoded to colour pictures for this plugin) and the settings
/// of `read_find_options`. It gives `{ expressions, faces, samples, durationMs, device }`. An
/// expression is `{ label, face, startMs, endMs, confidence, samples, box }`: a stretch where one
/// face keeps one expression, `box` being where the face was when it began, as shares of the
/// picture's width and height. A face is `{ face, startMs, endMs, samples }`. Faces are numbered
/// `face-1`, `face-2`... in the order they first show; the number says nothing of who it is.
pub fn find_faces(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let mut options = read_find_options(params)?;
    options.device = DeviceChoice::resolve(options.device, cancel.granted_device());
    let media = params
        .get("media")
        .ok_or_else(|| Failure::status(400, "media is missing: give a video"))?;
    let video = read_video_region(media)
        .map_err(|error| Failure::status(error.code, error.message))?
        .ok_or_else(|| Failure::status(400, "media must be a video Loomas decoded for this plugin (it asks for pictures)"))?;
    if video.format != PixelFormat::Rgb24 {
        return Err(Failure::status(400, "this plugin reads colour pictures (rgb24)"));
    }
    let models = locate_models(cancel)?;

    let mut sightings: Vec<Sighting> = Vec::new();
    let mut durations: Vec<u64> = Vec::new();
    let mut expressions = Vec::new();
    for (index, frame) in video.frames.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err(Failure::named("CANCELLED", "cancelled"));
        }
        if index % 20 == 0 {
            cancel.progress(&format!("{index} of {} pictures", video.frames.len()));
        }
        let Some(pixels) = video.pixels(index) else { continue };
        let faces = detect_faces(&models.faces, options.device, pixels, video.width, video.height, options.min_face_score)?;
        for face in faces {
            let expression = if face.width >= options.min_face_width {
                Some(classify_expression(&models.expressions, options.device, pixels, video.width, video.height, &face)?)
            } else {
                None
            };
            sightings.push(Sighting { time_ms: frame.pts_ms, face });
            durations.push(frame.duration_ms);
            expressions.push(expression);
        }
    }

    let numbers = track_faces(&sightings, TRACK_GAP_MS);
    let readings: Vec<Reading> = sightings
        .iter()
        .zip(&numbers)
        .zip(&durations)
        .zip(&expressions)
        .filter_map(|(((seen, face), duration), expression)| {
            expression.map(|expression| Reading {
                face: *face,
                time_ms: seen.time_ms,
                duration_ms: *duration,
                label: expression.label,
                confidence: expression.confidence,
                place: seen.face,
            })
        })
        .collect();
    let (width, height) = (video.width as f32, video.height as f32);
    let shares = |face: &Face| {
        json!({
            "x": round((face.x / width).clamp(0.0, 1.0)),
            "y": round((face.y / height).clamp(0.0, 1.0)),
            "width": round((face.width / width).min(1.0)),
            "height": round((face.height / height).min(1.0)),
        })
    };
    let described: Vec<Value> = expression_runs(&readings, options.gap_ms, options.min_samples)
        .iter()
        .map(|run| {
            json!({
                "label": EXPRESSIONS[run.label],
                "face": format!("face-{}", run.face + 1),
                "startMs": run.start_ms,
                "endMs": run.end_ms,
                "confidence": round(run.confidence),
                "samples": run.samples,
                "box": shares(&run.place),
            })
        })
        .collect();
    let face_count = numbers.iter().max().map_or(0, |highest| highest + 1);
    let faces: Vec<Value> = (0..face_count)
        .map(|face| {
            let seen: Vec<usize> = (0..sightings.len()).filter(|index| numbers[*index] == face).collect();
            let first = seen.first().copied().unwrap_or(0);
            let last = seen.last().copied().unwrap_or(0);
            json!({
                "face": format!("face-{}", face + 1),
                "startMs": sightings[first].time_ms,
                "endMs": sightings[last].time_ms + durations[last],
                "samples": seen.len(),
            })
        })
        .collect();
    Ok(json!({
        "expressions": described,
        "faces": faces,
        "samples": video.frames.len(),
        "durationMs": video.end_ms.saturating_sub(video.start_ms),
        "device": device_report().map(|report| report.to_value()),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_in_place_of_a_decoded_video_and_a_missing_media_are_400s() {
        let failure = find_faces(&json!({ "media": "a.mp4" }), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        let failure = find_faces(&json!({}), &Cancel::default()).unwrap_err();
        assert!(failure.message.contains("media is missing"));
    }
}
