use serde_json::Value;
use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{flag, number, text, whole_number, Failure};

use super::measure_options_contract::MeasureOptions;

/// The most segments one call takes.
const MAX_SEGMENTS: usize = 20_000;

/// The `segments` param: a list of objects, each with whole-number `startMs` and `endMs`, the end
/// after the start. Other fields are kept. Nothing given is `None`.
fn read_segments(params: &Value) -> Result<Option<Vec<Value>>, Failure> {
    let items = match params.get("segments") {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::Array(items)) if items.len() <= MAX_SEGMENTS => items,
        Some(_) => {
            return Err(Failure::status(
                400,
                format!("segments must be a list of at most {MAX_SEGMENTS} items with startMs and endMs"),
            ))
        }
    };
    for (index, item) in items.iter().enumerate() {
        let time = |key: &str| item.get(key).and_then(Value::as_u64);
        match (time("startMs"), time("endMs")) {
            (Some(start), Some(end)) if end > start => {}
            _ => {
                return Err(Failure::status(
                    400,
                    format!("segments[{index}] needs whole-number startMs and endMs, the end after the start"),
                ))
            }
        }
    }
    Ok(Some(items.clone()))
}

/// Reads the settings of a `measure` call, each with its default when it is not given. A value
/// out of its range is a 400 that says what was wrong.
pub fn read_measure_options(params: &Value) -> Result<MeasureOptions, Failure> {
    let device_text = text(params, "device", Some("cpu"), 12)?.unwrap_or_default();
    let device = DeviceChoice::parse(&device_text).ok_or_else(|| {
        Failure::status(400, format!("device must be auto, cpu, gpu or gpu:N, not {device_text:?}"))
    })?;
    Ok(MeasureOptions {
        segments: read_segments(params)?,
        window_ms: whole_number(params, "windowMs", 3_000, 1_000, 30_000)?,
        loud_db: number(params, "loudDb", 6.0, 2.0, 20.0)?,
        emotions: flag(params, "emotions", true)?,
        device,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn nothing_given_is_the_defaults() {
        let options = read_measure_options(&json!({})).unwrap();
        assert_eq!(options.segments, None);
        assert_eq!((options.window_ms, options.loud_db, options.emotions), (3_000, 6.0, true));
        assert_eq!(options.device, DeviceChoice::Cpu);
    }

    #[test]
    fn segments_keep_their_fields_and_bad_ones_say_which() {
        let options = read_measure_options(&json!({ "segments": [{ "startMs": 0, "endMs": 900, "text": "Hi" }] })).unwrap();
        assert_eq!(options.segments.unwrap()[0]["text"], "Hi");
        let failure = read_measure_options(&json!({ "segments": [{ "startMs": 0, "endMs": 900 }, { "startMs": 5, "endMs": 5 }] }))
            .unwrap_err();
        assert!(failure.message.contains("segments[1]"), "{}", failure.message);
        assert!(read_measure_options(&json!({ "loudDb": 1 })).unwrap_err().message.contains("loudDb"));
    }
}
