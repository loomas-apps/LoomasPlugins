use serde_json::Value;
use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{flag, number, text, whole_number, Failure};

use super::find_options_contract::FindOptions;

/// Reads the settings of a `find` call, each with its default when it is not given. A value out of
/// its range is a 400 that says what was wrong.
pub fn read_find_options(params: &Value) -> Result<FindOptions, Failure> {
    let device_text = text(params, "device", Some("cpu"), 12)?.unwrap_or_default();
    let device = DeviceChoice::parse(&device_text).ok_or_else(|| {
        Failure::status(400, format!("device must be auto, cpu, gpu or gpu:N, not {device_text:?}"))
    })?;
    let max_speakers = whole_number(params, "maxSpeakers", 0, 0, 50)? as usize;
    let min_similarity = number(params, "minSimilarity", 0.5, 0.1, 1.0)? as f32;
    let learn_similarity = number(params, "learnSimilarity", 0.65, 0.1, 1.0)? as f32;
    if learn_similarity < min_similarity {
        return Err(Failure::status(
            400,
            format!(
                "learnSimilarity ({learn_similarity}) must not be below minSimilarity ({min_similarity}): a pattern only learns from a voice it named"
            ),
        ));
    }
    Ok(FindOptions {
        max_speakers: (max_speakers > 0).then_some(max_speakers),
        min_similarity,
        learn: flag(params, "learn", true)?,
        learn_similarity,
        files: flag(params, "files", true)?,
        sample_ms: whole_number(params, "sampleMs", 8_000, 2_000, 30_000)?,
        device,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn nothing_given_is_the_defaults() {
        let options = read_find_options(&json!({})).unwrap();
        assert_eq!(options.max_speakers, None);
        assert_eq!((options.min_similarity, options.learn_similarity), (0.5, 0.65));
        assert!(options.learn && options.files);
        assert_eq!((options.sample_ms, options.device), (8_000, DeviceChoice::Cpu));
    }

    #[test]
    fn values_out_of_range_say_what_was_wrong() {
        let wrong = |params| read_find_options(&params).unwrap_err().message;
        assert!(wrong(json!({ "maxSpeakers": 51 })).contains("maxSpeakers"));
        assert!(wrong(json!({ "device": "tpu" })).contains("device"));
        assert!(wrong(json!({ "minSimilarity": 0.7, "learnSimilarity": 0.6 })).contains("must not be below"));
        assert_eq!(read_find_options(&json!({ "maxSpeakers": 3 })).unwrap().max_speakers, Some(3));
    }
}
