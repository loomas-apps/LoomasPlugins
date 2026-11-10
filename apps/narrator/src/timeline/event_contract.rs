/// Something found in a video with a start and an end, in milliseconds of the source, and what
/// the narrator reads of it. Each kind comes from the output of one plugin.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    /// When it starts.
    pub start_ms: u64,
    /// When it ends.
    pub end_ms: u64,
    /// What it is.
    pub kind: Kind,
}

/// What an event is.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// A sentence of speech-to-text: its words.
    Sentence { text: String },
    /// A turn of speakers: the name or id of the voice.
    Turn { speaker: String },
    /// A stretch measured by voice-tone: `low`, `normal` or `loud`, and the emotion of the voice
    /// with its confidence.
    Tone { loudness: Option<String>, emotion: Option<String>, confidence: f32 },
    /// A stretch of face-emotion: the face and its expression with its confidence.
    Expression { face: String, label: String, confidence: f32 },
    /// A movement: what moved.
    Movement { label: String },
    /// Something in view: a phrase of visual-phrases or a class of coco-objects.
    Scene { what: String },
}

impl Event {
    /// How many milliseconds this event and the stretch `start..end` have in common.
    pub fn overlap(&self, start: u64, end: u64) -> u64 {
        self.end_ms.min(end).saturating_sub(self.start_ms.max(start))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_overlap_is_the_time_in_common() {
        let event = Event { start_ms: 1_000, end_ms: 3_000, kind: Kind::Scene { what: "a cat".into() } };
        assert_eq!(event.overlap(2_000, 5_000), 1_000);
        assert_eq!(event.overlap(3_000, 5_000), 0);
        assert_eq!(event.overlap(0, 10_000), 2_000);
    }
}
