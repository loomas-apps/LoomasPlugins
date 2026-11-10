use crate::segment::{Window, FRAME_SPAN, FRAME_STEP, RATE, STEP, WINDOW};

/// A stretch of time one voice speaks.
#[derive(Debug, Clone, PartialEq)]
pub struct Turn {
    /// The voice, as numbered by the clustering.
    pub voice: usize,
    /// Start in milliseconds from the start of the sound.
    pub start_ms: u64,
    /// End in milliseconds from the start of the sound.
    pub end_ms: u64,
    /// The mean confidence of the speech model over the frames of the turn.
    pub confidence: f32,
}

/// How short a pause inside a turn can be before it splits it.
pub const GAP_MS: u64 = 300;
/// How short a turn can be before it is dropped.
pub const MIN_TURN_MS: u64 = 250;

/// The samples of the sound window `index` (of `count`, starting at `start`) speaks for: the
/// middle half of it, and from the start of the sound for the first, to its end for the last.
pub fn owned_range(index: usize, count: usize, start: usize) -> (usize, usize) {
    let margin = (WINDOW - STEP) / 2;
    let from = if index == 0 { 0 } else { start + margin };
    let to = if index + 1 == count { usize::MAX } else { start + WINDOW - margin };
    (from, to)
}

/// The middle sample of frame `frame` of a window that starts at `start`.
pub fn frame_middle(start: usize, frame: usize) -> usize {
    start + frame * FRAME_STEP + FRAME_SPAN / 2
}

fn to_ms(sample: usize) -> u64 {
    sample as u64 * 1000 / u64::from(RATE)
}

/// The turns of each voice: every window speaks for the middle of its time (see `owned_range`),
/// a frame where a local speaker talks counts for the voice `voices[window][local][frame]` says,
/// and frames of one voice closer than `GAP_MS` join into a turn. Turns shorter than
/// `MIN_TURN_MS` are dropped; times stop at `length` samples. Turns come in the order they start.
pub fn build_turns(windows: &[Window], voices: &[[Vec<Option<usize>>; 3]], length: usize) -> Vec<Turn> {
    let mut frames: Vec<(usize, u64, u64, f32)> = Vec::new();
    for (index, window) in windows.iter().enumerate() {
        let (from, to) = owned_range(index, windows.len(), window.start);
        for (number, label) in window.frames.iter().enumerate() {
            let middle = frame_middle(window.start, number);
            if middle < from || middle >= to || middle >= length {
                continue;
            }
            let start = middle.saturating_sub(FRAME_STEP / 2);
            let end = (middle + FRAME_STEP / 2).min(length);
            for (talks, frame_voices) in label.speakers.iter().zip(&voices[index]) {
                let voice = frame_voices.get(number).copied().flatten();
                if let (true, Some(voice)) = (talks, voice) {
                    frames.push((voice, to_ms(start), to_ms(end), label.confidence));
                }
            }
        }
    }
    frames.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut turns: Vec<Turn> = Vec::new();
    let mut weights: Vec<f32> = Vec::new();
    for (voice, start, end, confidence) in frames {
        match turns.last_mut() {
            Some(turn) if turn.voice == voice && start <= turn.end_ms + GAP_MS => {
                let weight = weights.last_mut().expect("one weight a turn");
                turn.confidence = (turn.confidence * *weight + confidence) / (*weight + 1.0);
                *weight += 1.0;
                turn.end_ms = turn.end_ms.max(end);
            }
            _ => {
                turns.push(Turn { voice, start_ms: start, end_ms: end, confidence });
                weights.push(1.0);
            }
        }
    }
    turns.retain(|turn| turn.end_ms - turn.start_ms >= MIN_TURN_MS);
    turns.sort_by(|a, b| a.start_ms.cmp(&b.start_ms).then(a.voice.cmp(&b.voice)));
    turns
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment::FrameLabel;

    fn window(start: usize, talks: impl Fn(usize) -> [bool; 3]) -> Window {
        let count = (WINDOW - FRAME_SPAN) / FRAME_STEP + 1;
        Window {
            start,
            frames: (0..count)
                .map(|frame| FrameLabel {
                    speakers: talks(frame_middle(start, frame)),
                    confidence: 0.9,
                })
                .collect(),
        }
    }

    #[test]
    fn windows_share_the_time_without_gaps_or_overlaps() {
        assert_eq!(owned_range(0, 3, 0), (0, 120_000));
        assert_eq!(owned_range(1, 3, STEP), (120_000, 200_000));
        assert_eq!(owned_range(2, 3, 2 * STEP), (200_000, usize::MAX));
        assert_eq!(owned_range(0, 1, 0), (0, usize::MAX));
    }

    #[test]
    fn one_voice_across_two_windows_is_one_turn_and_a_long_pause_splits_it() {
        // Talks from 2 s to 9 s and from 11 s to 13 s; the second window numbers its speakers
        // differently, and the clustering said both are voice 0.
        let talks = |sample: usize| {
            let second = sample as f32 / 16_000.0;
            (2.0..9.0).contains(&second) || (11.0..13.0).contains(&second)
        };
        let windows = [
            window(0, |s| [talks(s), false, false]),
            window(STEP, |s| [false, talks(s), false]),
        ];
        let frames = windows[0].frames.len();
        let every = |voice: Option<usize>| vec![voice; frames];
        let voices = [
            [every(Some(0)), every(None), every(None)],
            [every(None), every(Some(0)), every(None)],
        ];
        let turns = build_turns(&windows, &voices, 15 * 16_000);
        assert_eq!(turns.len(), 2, "{turns:?}");
        assert!(turns[0].start_ms.abs_diff(2_000) < 30 && turns[0].end_ms.abs_diff(9_000) < 30, "{turns:?}");
        assert!(turns[1].start_ms.abs_diff(11_000) < 30 && turns[1].end_ms.abs_diff(13_000) < 30);
        assert!((turns[0].confidence - 0.9).abs() < 1e-5);
    }

    #[test]
    fn a_local_speaker_with_no_voice_and_a_blip_are_dropped() {
        let windows = [window(0, |s| [s < 2_000, s >= 32_000, false])];
        let frames = windows[0].frames.len();
        let turns = build_turns(&windows, &[[vec![Some(0); frames], vec![None; frames], vec![]]], WINDOW);
        assert!(turns.is_empty(), "{turns:?}");
    }
}
