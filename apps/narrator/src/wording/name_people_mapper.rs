use std::collections::HashMap;

/// Gives each speaker the name the narration uses: a name a person gave (anything that is not a
/// `voice-N` id) stays, and the ids become "Person A", "Person B"... in the order of `speakers`
/// (the order they first speak), skipping letters a name already uses.
pub fn name_people(speakers: &[String]) -> HashMap<String, String> {
    let is_id = |speaker: &str| {
        speaker.strip_prefix("voice-").is_some_and(|number| !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()))
    };
    let mut names = HashMap::new();
    let mut letters = ('A'..='Z').map(|letter| format!("Person {letter}")).chain((27..).map(|n| format!("Person {n}")));
    for speaker in speakers {
        if names.contains_key(speaker) {
            continue;
        }
        let name = if is_id(speaker) {
            letters.find(|candidate| !speakers.contains(candidate)).expect("names never run out")
        } else {
            speaker.clone()
        };
        names.insert(speaker.clone(), name);
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_become_letters_in_order_and_names_stay() {
        let speakers: Vec<String> = ["voice-3", "John", "voice-1", "voice-3"].map(String::from).to_vec();
        let names = name_people(&speakers);
        assert_eq!(names["voice-3"], "Person A");
        assert_eq!(names["John"], "John");
        assert_eq!(names["voice-1"], "Person B");
    }
}
