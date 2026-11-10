use serde_json::Value;
use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{number, text, whole_number, Failure};

use super::find_options_contract::FindOptions;

/// Reads the settings of a `find` call, each with its default when it is not given. A value out
/// of its range is a 400 that says what was wrong.
pub fn read_find_options(params: &Value) -> Result<FindOptions, Failure> {
    let device_text = text(params, "device", Some("auto"), 12)?.unwrap_or_default();
    let device = DeviceChoice::parse(&device_text).ok_or_else(|| {
        Failure::status(400, format!("device must be auto, cpu, gpu or gpu:N, not {device_text:?}"))
    })?;
    Ok(FindOptions {
        min_face_score: number(params, "minFaceScore", 0.7, 0.3, 1.0)? as f32,
        min_face_width: whole_number(params, "minFaceWidth", 24, 8, 400)? as f32,
        gap_ms: whole_number(params, "gapMs", 1_000, 0, 600_000)?,
        min_samples: whole_number(params, "minSamples", 1, 1, 100)? as usize,
        device,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn nothing_given_is_the_defaults_and_a_value_out_of_range_says_which() {
        let options = read_find_options(&json!({})).unwrap();
        assert_eq!((options.min_face_score, options.min_face_width), (0.7, 24.0));
        assert_eq!((options.gap_ms, options.min_samples, options.device), (1_000, 1, DeviceChoice::Auto));
        assert!(read_find_options(&json!({ "minFaceScore": 0.1 })).unwrap_err().message.contains("minFaceScore"));
    }
}
