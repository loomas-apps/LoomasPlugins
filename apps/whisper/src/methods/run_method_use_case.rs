use loomasplugins_plugin_sdk::{Cancel, Failure};
use serde_json::Value;

use super::transcribe_use_case::transcribe;

/// Runs an action of this plugin: `transcribe` (the sentences said, with their times).
pub fn run_method(action: &str, params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    match action {
        "transcribe" => transcribe(params, cancel),
        other => Err(Failure::status(
            404,
            format!("this plugin has no action {other:?} (it has transcribe)"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn an_action_it_does_not_have_is_a_404_and_a_call_without_media_a_400() {
        let failure = run_method("find", &json!({}), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(404)));
        let failure = run_method("transcribe", &json!({}), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(400)));
    }
}
