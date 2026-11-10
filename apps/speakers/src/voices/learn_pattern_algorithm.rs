use crate::voiceprint::unit;

use super::voice_pattern_contract::VoicePattern;

/// A pattern weighs at most this many seconds against new speech, so it keeps following a voice
/// that changes (age, a cold, another microphone) instead of being fixed by its first hours.
pub const MEMORY_SECONDS: f64 = 600.0;

/// The pattern after hearing `seconds` more of its voice with the print `heard`: the mean of the
/// old print and the new one, each weighed by its seconds (the old at most `MEMORY_SECONDS`).
/// Its id and name stay.
pub fn learn_pattern(pattern: &VoicePattern, heard: &[f32], seconds: f64) -> VoicePattern {
    let old = pattern.seconds.min(MEMORY_SECONDS) as f32;
    let new = seconds.max(0.0) as f32;
    let print = if old + new == 0.0 {
        pattern.print.clone()
    } else {
        unit(
            pattern
                .print
                .iter()
                .zip(heard)
                .map(|(was, now)| was * old + now * new)
                .collect(),
        )
    };
    VoicePattern { print, seconds: pattern.seconds + seconds.max(0.0), ..pattern.clone() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_print_moves_towards_new_speech_by_its_share_of_the_seconds() {
        let pattern = VoicePattern { id: "voice-1".into(), name: "John".into(), seconds: 10.0, print: vec![1.0, 0.0] };
        let learned = learn_pattern(&pattern, &[0.0, 1.0], 10.0);
        assert!((learned.print[0] - learned.print[1]).abs() < 1e-6);
        assert_eq!((learned.seconds, learned.name.as_str()), (20.0, "John"));
    }

    #[test]
    fn an_old_pattern_still_learns() {
        let pattern = VoicePattern { id: "v".into(), name: String::new(), seconds: 100_000.0, print: vec![1.0, 0.0] };
        let learned = learn_pattern(&pattern, &[0.0, 1.0], 600.0);
        assert!((learned.print[0] - learned.print[1]).abs() < 1e-6);
    }
}
