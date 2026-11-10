use serde_json::{json, Value};
use loomasplugins_plugin_sdk::{number, whole_number, Failure};

use crate::timeline::{link_faces, Event, Kind};
use crate::wording::{describe_line, describe_quiet, face_phrase, movement_clause, name_people, voice_phrase, Said};

use super::read_events_mapper::read_events;

/// The most things in view one line names.
const MAX_SCENES: usize = 3;

/// The event of `events` that has the most time in common with `start..end` and that `wanted`
/// accepts, if any has some.
fn most_overlapping(events: &[Event], start: u64, end: u64, wanted: impl Fn(&Kind) -> bool) -> Option<&Event> {
    events
        .iter()
        .filter(|event| wanted(&event.kind) && event.overlap(start, end) > 0)
        .max_by_key(|event| event.overlap(start, end))
}

/// The distinct texts `pick` takes from the events overlapping `start..end`, in the order they
/// start.
fn overlapping_texts(events: &[Event], start: u64, end: u64, pick: impl Fn(&Kind) -> Option<String>) -> Vec<String> {
    let mut texts: Vec<String> = Vec::new();
    for event in events.iter().filter(|event| event.overlap(start, end) > 0) {
        if let Some(text) = pick(&event.kind) {
            if !texts.contains(&text) {
                texts.push(text);
            }
        }
    }
    texts
}

fn movement_of(kind: &Kind) -> Option<String> {
    match kind {
        Kind::Movement { label } => Some(movement_clause(label)),
        _ => None,
    }
}

fn scene_of(kind: &Kind) -> Option<String> {
    match kind {
        Kind::Scene { what } => Some(what.clone()),
        _ => None,
    }
}

/// The `narrate` action: what other plugins found in a video, written as a timeline of sentences.
///
/// Params: the lists `sentences` (speech-to-text), `turns` (speakers), `tones` (voice-tone),
/// `expressions` (face-emotion), `movements` (movement) and `scenes` (visual-phrases or
/// coco-objects), each as its plugin gives it and each optional; `minConfidence` (default 0.5),
/// below which an emotion or an expression is not written; and `answerWithinMs` (default 3000), how
/// soon after another person's question a line counts as its answer. It gives `{ lines, text,
/// people }`: a line is `{ startMs, endMs, text, speaker }` for each sentence (or each turn when
/// there are no sentences) and for each movement or thing in view while nobody speaks; `text` is
/// the lines one after another; `people` says what each speaker is called and which face was taken
/// for theirs.
pub fn narrate(params: &Value) -> Result<Value, Failure> {
    let min_confidence = number(params, "minConfidence", 0.5, 0.0, 1.0)? as f32;
    let answer_within_ms = whole_number(params, "answerWithinMs", 3_000, 0, 60_000)?;
    let mut events = read_events(params)?;
    events.sort_by_key(|event| (event.start_ms, event.end_ms));

    let has_sentences = events.iter().any(|event| matches!(event.kind, Kind::Sentence { .. }));
    let units: Vec<&Event> = events
        .iter()
        .filter(|event| match event.kind {
            Kind::Sentence { .. } => true,
            Kind::Turn { .. } => !has_sentences,
            _ => false,
        })
        .collect();
    let speaker_of = |unit: &Event| -> Option<String> {
        match &unit.kind {
            Kind::Turn { speaker } => Some(speaker.clone()),
            _ => match most_overlapping(&events, unit.start_ms, unit.end_ms, |kind| matches!(kind, Kind::Turn { .. })) {
                Some(Event { kind: Kind::Turn { speaker }, .. }) => Some(speaker.clone()),
                _ => None,
            },
        }
    };
    let speakers: Vec<Option<String>> = units.iter().map(|unit| speaker_of(unit)).collect();
    let names = name_people(&speakers.iter().flatten().cloned().collect::<Vec<_>>());
    let faces = link_faces(&events);

    let mut lines: Vec<(u64, u64, String, Option<String>)> = Vec::new();
    for (index, unit) in units.iter().enumerate() {
        let (start, end) = (unit.start_ms, unit.end_ms);
        let speaker = speakers[index].clone();
        let text = match &unit.kind {
            Kind::Sentence { text } => Some(text.clone()),
            _ => None,
        };
        let answers = index > 0 && {
            let before = units[index - 1];
            let asked = matches!(&before.kind, Kind::Sentence { text } if text.trim_end().ends_with('?'));
            asked && speakers[index - 1] != speaker && start.saturating_sub(before.end_ms) <= answer_within_ms
        };
        let tone = most_overlapping(&events, start, end, |kind| matches!(kind, Kind::Tone { .. }));
        let (loudness, voice) = match tone.map(|event| &event.kind) {
            Some(Kind::Tone { loudness, emotion, confidence }) => (
                loudness.clone(),
                emotion.as_deref().filter(|_| *confidence >= min_confidence).and_then(voice_phrase),
            ),
            _ => (None, None),
        };
        let face = speaker.as_ref().and_then(|speaker| faces.get(speaker)).and_then(|face| {
            let seen = most_overlapping(&events, start, end, |kind| matches!(kind, Kind::Expression { face: which, .. } if which == face));
            match seen.map(|event| &event.kind) {
                Some(Kind::Expression { label, confidence, .. }) if *confidence >= min_confidence => face_phrase(label),
                _ => None,
            }
        });
        let said = Said {
            speaker: speaker.as_ref().map_or_else(|| "Someone".to_string(), |speaker| names[speaker].clone()),
            text,
            answers,
            loudness,
            voice,
            face,
        };
        let movements = overlapping_texts(&events, start, end, movement_of);
        let mut scenes = overlapping_texts(&events, start, end, scene_of);
        scenes.truncate(MAX_SCENES);
        lines.push((start, end, describe_line(&said, &movements, &scenes), speaker.map(|speaker| names[&speaker].clone())));
    }
    for event in &events {
        let covered = units.iter().any(|unit| event.overlap(unit.start_ms, unit.end_ms) > 0);
        if covered {
            continue;
        }
        let quiet = match (movement_of(&event.kind), scene_of(&event.kind)) {
            (Some(movement), _) => describe_quiet(&[movement], &[]),
            (_, Some(scene)) => describe_quiet(&[], &[scene]),
            _ => continue,
        };
        lines.push((event.start_ms, event.end_ms, quiet, None));
    }
    lines.sort_by_key(|line| (line.0, line.1));

    let people: Vec<Value> = {
        let mut seen: Vec<&String> = Vec::new();
        for speaker in speakers.iter().flatten() {
            if !seen.contains(&speaker) {
                seen.push(speaker);
            }
        }
        seen.iter()
            .map(|speaker| json!({ "speaker": speaker, "name": names[*speaker], "face": faces.get(*speaker) }))
            .collect()
    };
    let text = lines.iter().map(|line| line.2.as_str()).collect::<Vec<_>>().join("\n");
    Ok(json!({
        "lines": lines
            .into_iter()
            .map(|(start, end, text, speaker)| json!({ "startMs": start, "endMs": end, "text": text, "speaker": speaker }))
            .collect::<Vec<_>>(),
        "text": text,
        "people": people,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The example of the issue: two people, a question, an answer, and an angry shout.
    fn scene() -> Value {
        json!({
            "sentences": [
                { "startMs": 0, "endMs": 2000, "text": "What is this doing here?" },
                { "startMs": 2500, "endMs": 6000, "text": "I left it there yesterday because it ripped." },
                { "startMs": 7000, "endMs": 10000, "text": "I already told you not to leave this mess in the house!" }
            ],
            "turns": [
                { "startMs": 0, "endMs": 2100, "label": "voice-1" },
                { "startMs": 2400, "endMs": 6100, "label": "voice-2" },
                { "startMs": 6900, "endMs": 10100, "label": "voice-1" }
            ],
            "tones": [
                { "startMs": 0, "endMs": 2100, "loudness": "normal", "emotion": "neutral", "emotionConfidence": 0.8 },
                { "startMs": 2400, "endMs": 6100, "loudness": "low", "emotion": "sad", "emotionConfidence": 0.4 },
                { "startMs": 6900, "endMs": 10100, "loudness": "loud", "emotion": "angry", "emotionConfidence": 0.9 }
            ],
            "expressions": [
                { "startMs": 0, "endMs": 6500, "face": "face-1", "label": "neutral", "confidence": 0.9 },
                { "startMs": 6500, "endMs": 10500, "face": "face-1", "label": "angry", "confidence": 0.8 }
            ],
            "movements": [
                { "startMs": 8000, "endMs": 8600, "label": "fast arm movement", "person": 1 },
                { "startMs": 12000, "endMs": 12500, "label": "falling", "person": 2 }
            ],
            "scenes": [{ "startMs": 0, "endMs": 10500, "phrase": "a person holding trousers" }]
        })
    }

    #[test]
    fn the_example_of_the_issue_reads_as_a_narration() {
        let narration = narrate(&scene()).unwrap();
        let lines: Vec<&str> = narration["lines"].as_array().unwrap().iter().map(|line| line["text"].as_str().unwrap()).collect();
        assert_eq!(lines[0], "Person A asked \u{201c}What is this doing here?\u{201d} On screen: a person holding trousers.");
        assert_eq!(lines[1], "Person B answered quietly \u{201c}I left it there yesterday because it ripped.\u{201d} On screen: a person holding trousers.");
        assert_eq!(
            lines[2],
            "Person A shouted in an angry voice, looking angry: \u{201c}I already told you not to leave this mess in the house!\u{201d} At the same time, a person moves an arm fast. On screen: a person holding trousers."
        );
        assert_eq!(lines[3], "A person falls.");
        assert_eq!(narration["people"][0]["face"], "face-1");
        assert_eq!(narration["people"][1]["face"], Value::Null);
    }

    #[test]
    fn turns_without_sentences_are_spoken_and_names_stay() {
        let narration = narrate(&json!({ "turns": [{ "startMs": 0, "endMs": 1000, "label": "John" }] })).unwrap();
        assert_eq!(narration["text"], "John spoke.");
    }

    #[test]
    fn nothing_given_is_an_empty_narration() {
        let narration = narrate(&json!({})).unwrap();
        assert_eq!(narration["lines"], json!([]));
        assert_eq!(narration["text"], "");
    }
}
