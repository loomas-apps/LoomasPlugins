use serde_json::Value;
use loomasplugins_plugin_sdk::{Cancel, Failure};

use super::narrate_use_case::narrate;

/// Runs an action of this plugin: `narrate` (the findings written as sentences).
pub fn run_method(action: &str, params: &Value, _cancel: &Cancel) -> Result<Value, Failure> {
    match action {
        "narrate" => narrate(params),
        other => Err(Failure::status(
            404,
            format!("this plugin has no action {other:?} (it has narrate)"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn an_action_it_does_not_have_is_a_404_that_names_the_one_it_has() {
        let failure = run_method("find", &json!({}), &Cancel::default()).unwrap_err();
        assert_eq!(failure.code, Some(json!(404)));
        assert!(failure.message.contains("it has narrate"));
    }
}
