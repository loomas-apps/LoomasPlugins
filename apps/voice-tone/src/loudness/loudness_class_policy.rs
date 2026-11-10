/// The middle of the levels given (the mean of the two middle ones for an even count), the level a
/// video's speech usually has. `None` when there is no level.
pub fn reference_level(levels: &[f64]) -> Option<f64> {
    let mut sorted: Vec<f64> = levels.iter().copied().filter(|level| level.is_finite()).collect();
    if sorted.is_empty() {
        return None;
    }
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    Some(if sorted.len().is_multiple_of(2) { (sorted[middle - 1] + sorted[middle]) / 2.0 } else { sorted[middle] })
}

/// Whether speech at `level` is `low`, `normal` or `loud` for a video whose speech is usually at
/// `reference`: `loud` at least `step` dB above it, `low` at least `step` dB below.
pub fn loudness_class(level: f64, reference: f64, step: f64) -> &'static str {
    if level >= reference + step {
        "loud"
    } else if level <= reference - step {
        "low"
    } else {
        "normal"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reference_is_the_middle_level() {
        assert_eq!(reference_level(&[-30.0, -10.0, -20.0]), Some(-20.0));
        assert_eq!(reference_level(&[-30.0, -20.0]), Some(-25.0));
        assert_eq!(reference_level(&[]), None);
    }

    #[test]
    fn speech_is_loud_or_low_only_a_step_away_from_the_reference() {
        assert_eq!(loudness_class(-14.0, -20.0, 6.0), "loud");
        assert_eq!(loudness_class(-15.0, -20.0, 6.0), "normal");
        assert_eq!(loudness_class(-26.0, -20.0, 6.0), "low");
    }
}
