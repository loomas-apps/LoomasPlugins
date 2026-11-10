use loomasplugins_plugin_sdk::{flag, text, whole_number, Failure};
use serde_json::Value;

use crate::device::DeviceChoice;
use crate::models::{model_named, model_names};

use super::transcribe_options_contract::TranscribeOptions;

/// The model used when the call names none.
pub const DEFAULT_MODEL: &str = "base";

/// Reads the settings of a `transcribe` call, each with its default when it is not given. A model,
/// a language or a device that is not one is a 400 that says what was wrong.
pub fn read_transcribe_options(params: &Value) -> Result<TranscribeOptions, Failure> {
    let model = text(params, "model", Some(DEFAULT_MODEL), 40)?.unwrap_or_default();
    if model_named(&model).is_none() {
        return Err(Failure::status(
            400,
            format!(
                "there is no model {model:?}: choose one of {}",
                model_names().join(", ")
            ),
        ));
    }
    let language = text(params, "language", None, 10)?
        .filter(|code| !code.trim().is_empty() && code != "auto");
    if let Some(code) = &language {
        if whisper_rs::get_lang_id(code).is_none() {
            return Err(Failure::status(
                400,
                format!("{code:?} is not a language code Whisper knows (en, pt, de...)"),
            ));
        }
    }
    let device_text = text(params, "device", Some("auto"), 10)?.unwrap_or_default();
    let device = DeviceChoice::parse(&device_text).ok_or_else(|| {
        Failure::status(
            400,
            format!("device must be auto, cpu, gpu or gpu:N, not {device_text:?}"),
        )
    })?;
    Ok(TranscribeOptions {
        model,
        language,
        words: flag(params, "words", false)?,
        skip_silence: flag(params, "skipSilence", false)?,
        beam_size: whole_number(params, "beamSize", 5, 1, 10)?,
        device,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn defaults_and_wrong_settings() {
        let options = read_transcribe_options(&json!({})).unwrap();
        assert_eq!(
            (options.model.as_str(), options.language, options.beam_size),
            (DEFAULT_MODEL, None, 5)
        );
        assert_eq!(
            read_transcribe_options(&json!({ "language": "pt" }))
                .unwrap()
                .language
                .as_deref(),
            Some("pt")
        );
        for (params, wanted) in [
            (json!({ "model": "huge" }), "no model"),
            (json!({ "language": "xx" }), "language code"),
            (json!({ "device": "vulkan" }), "device must be"),
            (json!({ "beamSize": 0 }), "beamSize"),
        ] {
            let failure = read_transcribe_options(&params).unwrap_err();
            assert!(failure.message.contains(wanted), "{}", failure.message);
        }
    }
}
