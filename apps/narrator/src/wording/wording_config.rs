/// How the emotion of a voice is written after what was said, or `None` for one that is not worth
/// saying (`neutral`, `calm`, or an emotion the narrator does not know).
pub fn voice_phrase(emotion: &str) -> Option<&'static str> {
    match emotion {
        "angry" => Some("in an angry voice"),
        "happy" => Some("in a happy voice"),
        "sad" => Some("in a sad voice"),
        "fear" => Some("in a frightened voice"),
        "surprise" => Some("in a surprised voice"),
        "disgust" => Some("in a disgusted voice"),
        _ => None,
    }
}

/// How the expression of a face is written, or `None` for `neutral` or one the narrator does not
/// know.
pub fn face_phrase(expression: &str) -> Option<&'static str> {
    match expression {
        "angry" => Some("looking angry"),
        "happy" => Some("looking happy"),
        "sad" => Some("looking sad"),
        "fear" => Some("looking frightened"),
        "surprise" => Some("looking surprised"),
        "disgust" => Some("looking disgusted"),
        _ => None,
    }
}

/// How a movement of the movement plugin is written as a clause: who moved is not known to the
/// narrator, so it is always "a person". A label it does not know is written as it is.
pub fn movement_clause(label: &str) -> String {
    match label {
        "fast arm movement" => "a person moves an arm fast".into(),
        "fast leg movement" => "a person moves a leg fast".into(),
        "falling" => "a person falls".into(),
        "person on the ground" => "a person is on the ground".into(),
        "striking motion towards another person" => "a person makes a striking motion towards another".into(),
        "two people close with fast movement" => "two people move fast close together".into(),
        other => format!("a person: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_and_unknown_feelings_are_not_written() {
        assert_eq!(voice_phrase("angry"), Some("in an angry voice"));
        assert_eq!(voice_phrase("neutral"), None);
        assert_eq!(face_phrase("calm"), None);
        assert_eq!(movement_clause("falling"), "a person falls");
        assert_eq!(movement_clause("waving"), "a person: waving");
    }
}
