/// What the narrator knows of one thing said.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Said {
    /// Who said it, as the narration names them ("Someone" when no turn covers it).
    pub speaker: String,
    /// The words, when speech-to-text gave them.
    pub text: Option<String>,
    /// Whether it answers a question another person asked just before.
    pub answers: bool,
    /// `low`, `normal` or `loud`, when voice-tone measured it.
    pub loudness: Option<String>,
    /// How the voice sounded ("in an angry voice"), when worth saying.
    pub voice: Option<&'static str>,
    /// How the speaker's face looked ("looking angry"), when worth saying.
    pub face: Option<&'static str>,
}

/// The verb for something said: asked (a question), answered (right after another person's
/// question) or said, made shouted or whispered by a loud or low voice ("asked loudly",
/// "answered quietly" for the others).
fn verb(said: &Said) -> String {
    let question = said.text.as_deref().is_some_and(|text| text.trim_end().ends_with('?'));
    let base = if question {
        "asked"
    } else if said.answers {
        "answered"
    } else {
        "said"
    };
    match (said.loudness.as_deref(), base) {
        (Some("loud"), "said") => "shouted".into(),
        (Some("low"), "said") => "whispered".into(),
        (Some("loud"), other) => format!("{other} loudly"),
        (Some("low"), other) => format!("{other} quietly"),
        (_, other) => other.into(),
    }
}

/// The line for something said: `Person A shouted in an angry voice, looking angry: "Get out."`
/// (a full stop after the quote only when the words have no end of their own). Without words:
/// `Person A spoke in an angry voice.` Then what happened at the same time:
/// ` At the same time, a person moves an arm fast.` and ` On screen: a person holding trousers.`
pub fn describe_line(said: &Said, movements: &[String], scenes: &[String]) -> String {
    let details: Vec<&str> = [said.voice, said.face].into_iter().flatten().collect();
    let mut line = match &said.text {
        Some(text) => {
            let text = text.trim();
            let lead = if details.is_empty() {
                format!("{} {}", said.speaker, verb(said))
            } else {
                format!("{} {} {}:", said.speaker, verb(said), details.join(", "))
            };
            let end = if text.ends_with(['.', '?', '!']) { "" } else { "." };
            format!("{lead} \u{201c}{text}\u{201d}{end}")
        }
        None if details.is_empty() => format!("{} spoke.", said.speaker),
        None => format!("{} spoke {}.", said.speaker, details.join(", ")),
    };
    if !movements.is_empty() {
        line.push_str(&format!(" At the same time, {}.", join(movements)));
    }
    if !scenes.is_empty() {
        line.push_str(&format!(" On screen: {}.", join(scenes)));
    }
    line
}

/// The line for something that happened while nobody spoke: `A person falls.` and `On screen: ...`.
pub fn describe_quiet(movements: &[String], scenes: &[String]) -> String {
    let mut parts = Vec::new();
    if !movements.is_empty() {
        let text = join(movements);
        let mut letters = text.chars();
        let first = letters.next().map(|letter| letter.to_uppercase().collect::<String>()).unwrap_or_default();
        parts.push(format!("{first}{}.", letters.as_str()));
    }
    if !scenes.is_empty() {
        parts.push(format!("On screen: {}.", join(scenes)));
    }
    parts.join(" ")
}

/// `a`, `a and b`, `a, b and c`.
fn join(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn said(text: Option<&str>) -> Said {
        Said { speaker: "Person A".into(), text: text.map(String::from), ..Said::default() }
    }

    #[test]
    fn a_loud_angry_line_is_shouted_with_the_voice_and_the_face() {
        let line = Said {
            loudness: Some("loud".into()),
            voice: Some("in an angry voice"),
            face: Some("looking angry"),
            ..said(Some("I already told you"))
        };
        assert_eq!(
            describe_line(&line, &["a person moves an arm fast".into()], &["a pair of trousers".into()]),
            "Person A shouted in an angry voice, looking angry: \u{201c}I already told you\u{201d}. At the same time, a person moves an arm fast. On screen: a pair of trousers."
        );
    }

    #[test]
    fn words_that_end_a_sentence_end_the_line_at_the_quote() {
        assert_eq!(describe_line(&said(Some("Fine.")), &[], &[]), "Person A said \u{201c}Fine.\u{201d}");
        assert_eq!(describe_line(&said(Some("Fine")), &[], &[]), "Person A said \u{201c}Fine\u{201d}.");
    }

    #[test]
    fn questions_are_asked_and_the_reply_is_answered() {
        assert!(describe_line(&said(Some("What is this?")), &[], &[]).starts_with("Person A asked"));
        let reply = Said { answers: true, loudness: Some("low".into()), ..said(Some("I left it")) };
        assert!(describe_line(&reply, &[], &[]).starts_with("Person A answered quietly"));
        assert!(describe_line(&Said { loudness: Some("low".into()), ..said(Some("Hush")) }, &[], &[]).contains("whispered"));
    }

    #[test]
    fn without_words_a_turn_is_spoken_and_quiet_events_are_their_own_line() {
        assert_eq!(describe_line(&said(None), &[], &[]), "Person A spoke.");
        assert_eq!(
            describe_quiet(&["a person falls".into()], &["a cat".into(), "a sofa".into()]),
            "A person falls. On screen: a cat and a sofa."
        );
    }
}
