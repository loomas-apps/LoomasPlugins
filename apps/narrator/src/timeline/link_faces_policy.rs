use std::collections::HashMap;

use super::event_contract::{Event, Kind};

/// A face is taken for a speaker's face when it is in view for at least this share of the time the
/// speaker talks.
const MIN_SHARE: f64 = 0.5;

/// Which face each speaker has: for every speaker and face, the share of the speaker's talking
/// time the face is in view (by its expressions); the highest shares are paired first, each face
/// and speaker once, and a share under one half pairs nothing. A guess from timing only: a person
/// who talks off screen while another is in view would be paired wrongly, so a speaker with two
/// faces near the same share is left without a face.
pub fn link_faces(events: &[Event]) -> HashMap<String, String> {
    let turns: Vec<(&str, u64, u64)> = events
        .iter()
        .filter_map(|event| match &event.kind {
            Kind::Turn { speaker } => Some((speaker.as_str(), event.start_ms, event.end_ms)),
            _ => None,
        })
        .collect();
    let faces: Vec<(&str, &Event)> = events
        .iter()
        .filter_map(|event| match &event.kind {
            Kind::Expression { face, .. } => Some((face.as_str(), event)),
            _ => None,
        })
        .collect();
    let mut talking: HashMap<&str, u64> = HashMap::new();
    let mut seen: HashMap<(&str, &str), u64> = HashMap::new();
    for (speaker, start, end) in &turns {
        *talking.entry(speaker).or_default() += end - start;
        for (face, event) in &faces {
            let common = event.overlap(*start, *end);
            if common > 0 {
                *seen.entry((speaker, face)).or_default() += common;
            }
        }
    }
    let mut pairs: Vec<(f64, &str, &str)> = seen
        .iter()
        .map(|((speaker, face), common)| (*common as f64 / talking[speaker].max(1) as f64, *speaker, *face))
        .filter(|(share, _, _)| *share >= MIN_SHARE)
        .collect();
    pairs.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(b.1)).then(a.2.cmp(b.2)));
    let mut linked = HashMap::new();
    let mut used = Vec::new();
    for (share, speaker, face) in &pairs {
        if linked.contains_key(*speaker) || used.contains(face) {
            continue;
        }
        let rival = pairs
            .iter()
            .any(|(other, who, which)| who == speaker && which != face && !used.contains(which) && share - other < 0.1);
        if !rival {
            linked.insert((*speaker).to_string(), (*face).to_string());
            used.push(*face);
        }
    }
    linked
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(speaker: &str, start_ms: u64, end_ms: u64) -> Event {
        Event { start_ms, end_ms, kind: Kind::Turn { speaker: speaker.into() } }
    }

    fn face(face: &str, start_ms: u64, end_ms: u64) -> Event {
        Event { start_ms, end_ms, kind: Kind::Expression { face: face.into(), label: "neutral".into(), confidence: 0.9 } }
    }

    #[test]
    fn a_speaker_gets_the_face_in_view_while_they_talk() {
        let events = [turn("Ann", 0, 4_000), turn("Bob", 5_000, 9_000), face("face-1", 0, 4_000), face("face-2", 5_000, 9_000)];
        let linked = link_faces(&events);
        assert_eq!(linked.get("Ann").map(String::as_str), Some("face-1"));
        assert_eq!(linked.get("Bob").map(String::as_str), Some("face-2"));
    }

    #[test]
    fn two_faces_in_view_the_whole_time_leave_the_speaker_without_a_face() {
        let events = [turn("Ann", 0, 4_000), face("face-1", 0, 4_000), face("face-2", 0, 4_000)];
        assert!(link_faces(&events).is_empty());
    }

    #[test]
    fn a_face_seen_for_little_of_the_talk_is_not_the_speakers() {
        let events = [turn("Ann", 0, 10_000), face("face-1", 0, 2_000)];
        assert!(link_faces(&events).is_empty());
    }
}
