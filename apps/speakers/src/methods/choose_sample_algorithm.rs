use crate::cluster::Turn;

/// The stretch of a voice to play to a person who names it, in milliseconds: the middle of its
/// longest turn that no other voice talks over (or of its longest turn, when every one is talked
/// over), at most `longest_ms` long. `None` when the voice has no turn.
pub fn choose_sample(turns: &[Turn], voice: usize, longest_ms: u64) -> Option<(u64, u64)> {
    let overlapped = |turn: &Turn| {
        turns
            .iter()
            .any(|other| other.voice != turn.voice && other.start_ms < turn.end_ms && turn.start_ms < other.end_ms)
    };
    let own = || turns.iter().filter(|turn| turn.voice == voice);
    let best = own()
        .filter(|turn| !overlapped(turn))
        .max_by_key(|turn| turn.end_ms - turn.start_ms)
        .or_else(|| own().max_by_key(|turn| turn.end_ms - turn.start_ms))?;
    let length = (best.end_ms - best.start_ms).min(longest_ms);
    let start = best.start_ms + (best.end_ms - best.start_ms - length) / 2;
    Some((start, start + length))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(voice: usize, start_ms: u64, end_ms: u64) -> Turn {
        Turn { voice, start_ms, end_ms, confidence: 0.9 }
    }

    #[test]
    fn the_middle_of_the_longest_clean_turn_is_played() {
        let turns = [turn(0, 0, 20_000), turn(1, 5_000, 6_000), turn(0, 30_000, 34_000)];
        assert_eq!(choose_sample(&turns, 0, 8_000), Some((30_000, 34_000)));
        assert_eq!(choose_sample(&turns, 1, 8_000), Some((5_000, 6_000)));
    }

    #[test]
    fn a_voice_always_talked_over_still_gets_its_longest_turn_cut_to_length() {
        let turns = [turn(0, 0, 20_000), turn(1, 1_000, 19_000)];
        assert_eq!(choose_sample(&turns, 0, 8_000), Some((6_000, 14_000)));
        assert_eq!(choose_sample(&turns, 2, 8_000), None);
    }
}
