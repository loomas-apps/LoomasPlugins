use serde_json::Value;
use loomasplugins_plugin_sdk::Failure;

use crate::timeline::{Event, Kind};

/// The most items all lists together may hold.
const MAX_ITEMS: usize = 100_000;

fn text_of(item: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| item.get(*key).and_then(Value::as_str)).map(str::to_string)
}

fn number_of(item: &Value, key: &str) -> f32 {
    item.get(key).and_then(Value::as_f64).unwrap_or(0.0) as f32
}

/// The kind of event an item of the list `list` is, or `None` when a field it needs is missing.
fn kind_of(list: &str, item: &Value) -> Option<Kind> {
    Some(match list {
        "sentences" => Kind::Sentence { text: text_of(item, &["text"])? },
        "turns" => Kind::Turn { speaker: text_of(item, &["label", "speaker", "voice"])? },
        "tones" => Kind::Tone {
            loudness: text_of(item, &["loudness"]),
            emotion: text_of(item, &["emotion"]),
            confidence: number_of(item, "emotionConfidence"),
        },
        "expressions" => Kind::Expression {
            face: text_of(item, &["face"])?,
            label: text_of(item, &["label"])?,
            confidence: number_of(item, "confidence"),
        },
        "movements" => Kind::Movement { label: text_of(item, &["label"])? },
        "scenes" => Kind::Scene { what: text_of(item, &["phrase", "label", "class"])? },
        _ => return None,
    })
}

/// Every event of the lists in `params` (`sentences`, `turns`, `tones`, `expressions`,
/// `movements`, `scenes`), each item as its plugin gives it. An item needs whole-number `startMs`
/// and `endMs` (the end not before the start) and the field its list is read for; a missing one is
/// a 400 that names the item. Lists not given are empty.
pub fn read_events(params: &Value) -> Result<Vec<Event>, Failure> {
    let mut events = Vec::new();
    for list in ["sentences", "turns", "tones", "expressions", "movements", "scenes"] {
        let items = match params.get(list) {
            None | Some(Value::Null) => continue,
            Some(Value::Array(items)) => items,
            Some(_) => return Err(Failure::status(400, format!("{list} must be a list"))),
        };
        for (index, item) in items.iter().enumerate() {
            let time = |key: &str| item.get(key).and_then(Value::as_u64);
            let (Some(start_ms), Some(end_ms)) = (time("startMs"), time("endMs")) else {
                return Err(Failure::status(400, format!("{list}[{index}] needs whole-number startMs and endMs")));
            };
            if end_ms < start_ms {
                return Err(Failure::status(400, format!("{list}[{index}] ends before it starts")));
            }
            let kind = kind_of(list, item).ok_or_else(|| {
                Failure::status(400, format!("{list}[{index}] lacks the field the narrator reads (see the README)"))
            })?;
            events.push(Event { start_ms, end_ms, kind });
            if events.len() > MAX_ITEMS {
                return Err(Failure::status(400, format!("the lists hold more than {MAX_ITEMS} items together")));
            }
        }
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn items_are_read_as_their_plugins_give_them() {
        let events = read_events(&json!({
            "sentences": [{ "startMs": 0, "endMs": 900, "text": "Hi.", "confidence": 0.9 }],
            "turns": [{ "startMs": 0, "endMs": 1000, "label": "John", "voice": "voice-1" }],
            "scenes": [{ "startMs": 0, "endMs": 5000, "phrase": "a kitchen" }, { "startMs": 0, "endMs": 5000, "class": "cup" }],
        }))
        .unwrap();
        assert_eq!(events.len(), 4);
        assert_eq!(events[1].kind, Kind::Turn { speaker: "John".into() });
        assert_eq!(events[3].kind, Kind::Scene { what: "cup".into() });
    }

    #[test]
    fn a_wrong_item_names_its_list_and_place() {
        let failure = read_events(&json!({ "sentences": [{ "startMs": 0, "endMs": 900 }] })).unwrap_err();
        assert!(failure.message.contains("sentences[0]"), "{}", failure.message);
        let failure = read_events(&json!({ "turns": [{ "startMs": 9, "endMs": 1, "label": "x" }] })).unwrap_err();
        assert!(failure.message.contains("ends before"));
        assert!(read_events(&json!({})).unwrap().is_empty());
    }
}
