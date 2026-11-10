use std::path::{Path, PathBuf};

use serde_json::Value;
use loomasplugins_plugin_sdk::Failure;

use super::voice_pattern_contract::VoicePattern;

/// The most patterns one call takes.
pub const MAX_PATTERNS: usize = 100;
/// The name of the pattern file inside a voice folder.
pub const PATTERN_FILE: &str = "voice.json";

/// The voice patterns of the `voices` param: each one a pattern as JSON, a stored file (with its
/// `file` path), or a path in text to a `voice.json` or to the folder that holds one. Nothing
/// given is no pattern. A file that is not there is a 404; a wrong pattern or two with the same id
/// is a 400.
pub fn read_voice_patterns(params: &Value) -> Result<Vec<VoicePattern>, Failure> {
    let items = match params.get("voices") {
        None | Some(Value::Null) => return Ok(Vec::new()),
        Some(Value::Array(items)) if items.len() <= MAX_PATTERNS => items,
        Some(_) => {
            return Err(Failure::status(
                400,
                format!("voices must be a list of at most {MAX_PATTERNS} voice patterns or their folders"),
            ))
        }
    };
    let mut patterns: Vec<VoicePattern> = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let pattern = match item {
            Value::Object(fields) if fields.contains_key("kind") => {
                VoicePattern::from_value(item, &format!("voices[{index}]"))?
            }
            Value::Object(fields) => match fields.get("file").and_then(Value::as_str) {
                Some(file) => read_pattern_file(Path::new(file))?,
                None => {
                    return Err(Failure::status(
                        400,
                        format!("voices[{index}] is neither a voice pattern nor a stored file"),
                    ))
                }
            },
            Value::String(path) => read_pattern_file(Path::new(path))?,
            other => {
                return Err(Failure::status(
                    400,
                    format!("voices[{index}] must be a voice pattern or a path, not {other}"),
                ))
            }
        };
        if patterns.iter().any(|known| known.id == pattern.id) {
            return Err(Failure::status(
                400,
                format!("two voice patterns have the id {:?}: give each voice once", pattern.id),
            ));
        }
        patterns.push(pattern);
    }
    Ok(patterns)
}

/// Reads the pattern at `path`, or in the `voice.json` of the folder `path`.
fn read_pattern_file(path: &Path) -> Result<VoicePattern, Failure> {
    let file: PathBuf = if path.is_dir() { path.join(PATTERN_FILE) } else { path.to_path_buf() };
    let text = std::fs::read_to_string(&file).map_err(|error| {
        let code = if error.kind() == std::io::ErrorKind::NotFound { 404 } else { 422 };
        Failure::status(code, format!("the voice pattern {} cannot be read: {error}", file.display()))
    })?;
    let value: Value = serde_json::from_str(&text).map_err(|error| {
        Failure::status(400, format!("the voice pattern {} is not JSON: {error}", file.display()))
    })?;
    VoicePattern::from_value(&value, &file.display().to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::voiceprint::PRINT_SIZE;

    fn pattern(id: &str) -> VoicePattern {
        let mut print = vec![0.0; PRINT_SIZE];
        print[1] = 1.0;
        VoicePattern { id: id.into(), name: String::new(), seconds: 3.0, print }
    }

    #[test]
    fn patterns_come_inline_as_a_file_or_as_the_folder_of_one() {
        let folder = std::env::temp_dir().join(format!("speakers-patterns-{}", std::process::id()));
        let voice_folder = folder.join("voice-2");
        std::fs::create_dir_all(&voice_folder).unwrap();
        std::fs::write(voice_folder.join(PATTERN_FILE), pattern("voice-2").to_value().to_string()).unwrap();
        std::fs::write(folder.join("three.json"), pattern("voice-3").to_value().to_string()).unwrap();

        let read = read_voice_patterns(&json!({ "voices": [
            pattern("voice-1").to_value(),
            voice_folder.to_string_lossy(),
            { "file": folder.join("three.json").to_string_lossy() },
        ] }))
        .unwrap();
        let ids: Vec<&str> = read.iter().map(|voice| voice.id.as_str()).collect();
        assert_eq!(ids, ["voice-1", "voice-2", "voice-3"]);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn nothing_given_is_no_pattern_and_bad_ones_say_why() {
        assert!(read_voice_patterns(&json!({})).unwrap().is_empty());
        let missing = read_voice_patterns(&json!({ "voices": ["no/such/voice.json"] })).unwrap_err();
        assert_eq!(missing.code, Some(json!(404)));
        let twice = read_voice_patterns(&json!({ "voices": [pattern("a").to_value(), pattern("a").to_value()] }))
            .unwrap_err();
        assert!(twice.message.contains("two voice patterns"), "{}", twice.message);
        assert_eq!(read_voice_patterns(&json!({ "voices": [5] })).unwrap_err().code, Some(json!(400)));
    }
}
