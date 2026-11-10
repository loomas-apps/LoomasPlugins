/// Samples in a frame of the level: 50 ms at 16 kHz.
const FRAME: usize = 800;
/// A frame quieter than this, in dB below full scale, is a pause and does not count.
const SILENCE_DB: f64 = -50.0;

fn decibels(energy: f64) -> f64 {
    10.0 * energy.max(1e-12).log10()
}

/// How loud the speech in `samples` (from -1 to 1) is, in dB below full scale: the mean energy of
/// its 50 ms frames that are louder than `SILENCE_DB`, so the pauses between words do not make a
/// sentence sound quieter. `None` when every frame is a pause (or there is no sound).
pub fn speech_level(samples: &[f32]) -> Option<f64> {
    let energies: Vec<f64> = samples
        .chunks(FRAME)
        .map(|frame| frame.iter().map(|s| f64::from(*s) * f64::from(*s)).sum::<f64>() / frame.len() as f64)
        .filter(|energy| decibels(*energy) >= SILENCE_DB)
        .collect();
    if energies.is_empty() {
        return None;
    }
    Some(decibels(energies.iter().sum::<f64>() / energies.len() as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(amplitude: f32, length: usize) -> Vec<f32> {
        (0..length).map(|n| amplitude * (n as f32 * 0.1).sin()).collect()
    }

    #[test]
    fn a_sine_at_half_scale_is_about_minus_9_db() {
        let level = speech_level(&tone(0.5, 16_000)).unwrap();
        assert!((level + 9.03).abs() < 0.1, "{level}");
    }

    #[test]
    fn pauses_do_not_make_speech_quieter() {
        let mut with_pause = tone(0.5, 8_000);
        with_pause.extend(vec![0.0; 16_000]);
        with_pause.extend(tone(0.5, 8_000));
        let level = speech_level(&with_pause).unwrap();
        assert!((level - speech_level(&tone(0.5, 16_000)).unwrap()).abs() < 0.1);
    }

    #[test]
    fn silence_has_no_level() {
        assert_eq!(speech_level(&[0.0; 8_000]), None);
        assert_eq!(speech_level(&[]), None);
    }
}
