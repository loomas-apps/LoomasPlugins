use serde_json::{json, Value};
use loomasplugins_plugin_sdk::Failure;

use crate::models::VOICE_PRINT_MODEL;
use crate::voiceprint::{unit, PRINT_SIZE};

/// What a voice pattern file says it is.
pub const PATTERN_KIND: &str = "loomas-voice";
/// The version of the pattern file. A reader refuses one it does not know.
pub const PATTERN_FORMAT: u64 = 1;

/// A voice as it is kept between runs (`voice.json`): its id, the name a person gave it (empty
/// until then), how many seconds of speech it was learned from, and its print.
#[derive(Debug, Clone, PartialEq)]
pub struct VoicePattern {
    /// The id the plugin gave the voice when it first heard it (`voice-1`).
    pub id: String,
    /// The name a person typed, or empty.
    pub name: String,
    /// The seconds of speech the print was learned from.
    pub seconds: f64,
    /// The voice print, of length 1.
    pub print: Vec<f32>,
}

impl VoicePattern {
    /// The name when a person gave one, the id otherwise: what the turns call the voice.
    pub fn label(&self) -> &str {
        if self.name.trim().is_empty() {
            &self.id
        } else {
            self.name.trim()
        }
    }

    /// The pattern as the JSON of `voice.json`. Print numbers keep 6 decimals.
    pub fn to_value(&self) -> Value {
        let print: Vec<f64> = self
            .print
            .iter()
            .map(|value| (f64::from(*value) * 1e6).round() / 1e6)
            .collect();
        json!({
            "kind": PATTERN_KIND,
            "format": PATTERN_FORMAT,
            "model": VOICE_PRINT_MODEL,
            "id": self.id,
            "name": self.name,
            "seconds": (self.seconds * 10.0).round() / 10.0,
            "print": print,
        })
    }

    /// Reads a pattern from the JSON of `voice.json`. `source` names where it came from in the
    /// error. A wrong kind, format, model or print is a 400 that says what was wrong.
    pub fn from_value(value: &Value, source: &str) -> Result<VoicePattern, Failure> {
        let wrong = |what: String| Failure::status(400, format!("{source} is not a voice pattern: {what}"));
        if value.get("kind").and_then(Value::as_str) != Some(PATTERN_KIND) {
            return Err(wrong(format!("its kind is not {PATTERN_KIND:?}")));
        }
        match value.get("format").and_then(Value::as_u64) {
            Some(PATTERN_FORMAT) => {}
            other => return Err(wrong(format!("format {other:?} is not one this plugin reads ({PATTERN_FORMAT})"))),
        }
        let model = value.get("model").and_then(Value::as_str).unwrap_or_default();
        if model != VOICE_PRINT_MODEL {
            return Err(Failure::status(
                400,
                format!(
                    "{source} was made by the model {model:?}, and this plugin uses {VOICE_PRINT_MODEL:?}: prints of two models cannot be compared, so run the video again to make a new pattern"
                ),
            ));
        }
        let id = value.get("id").and_then(Value::as_str).unwrap_or_default().to_string();
        if id.is_empty() || id.chars().count() > 100 {
            return Err(wrong("its id is missing or longer than 100 characters".to_string()));
        }
        let name = value.get("name").and_then(Value::as_str).unwrap_or_default().to_string();
        if name.chars().count() > 100 {
            return Err(wrong("its name is longer than 100 characters".to_string()));
        }
        let seconds = value.get("seconds").and_then(Value::as_f64).unwrap_or(0.0).max(0.0);
        let print: Option<Vec<f32>> = value.get("print").and_then(Value::as_array).map(|numbers| {
            numbers.iter().filter_map(Value::as_f64).map(|number| number as f32).collect()
        });
        let print = match print {
            Some(print) if print.len() == PRINT_SIZE && print.iter().all(|value| value.is_finite()) => unit(print),
            _ => return Err(wrong(format!("its print is not {PRINT_SIZE} numbers"))),
        };
        Ok(VoicePattern { id, name, seconds, print })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pattern() -> VoicePattern {
        let mut print = vec![0.0; PRINT_SIZE];
        print[0] = 1.0;
        VoicePattern { id: "voice-1".into(), name: "John".into(), seconds: 12.34, print }
    }

    #[test]
    fn a_pattern_reads_back_what_it_wrote() {
        let value = pattern().to_value();
        assert_eq!(value["kind"], PATTERN_KIND);
        let read = VoicePattern::from_value(&value, "voice.json").unwrap();
        assert_eq!(read.name, "John");
        assert_eq!(read.seconds, 12.3);
        assert_eq!(read.print, pattern().print);
    }

    #[test]
    fn the_label_is_the_name_or_else_the_id() {
        let mut voice = pattern();
        assert_eq!(voice.label(), "John");
        voice.name = "  ".into();
        assert_eq!(voice.label(), "voice-1");
    }

    #[test]
    fn a_pattern_of_another_model_or_a_short_print_is_refused_with_why() {
        let mut value = pattern().to_value();
        value["model"] = json!("ecapa");
        let failure = VoicePattern::from_value(&value, "a.json").unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
        assert!(failure.message.contains("cannot be compared"), "{}", failure.message);

        let mut value = pattern().to_value();
        value["print"] = json!([1, 2]);
        assert!(VoicePattern::from_value(&value, "a.json").unwrap_err().message.contains("256 numbers"));

        assert!(VoicePattern::from_value(&json!({}), "a.json").unwrap_err().message.contains("kind"));
    }
}
