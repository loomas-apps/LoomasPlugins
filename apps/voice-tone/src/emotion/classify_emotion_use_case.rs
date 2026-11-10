use std::path::Path;

use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::Failure;

use super::emotion_labels_config::EMOTIONS;
use super::emotion_store::run_emotion_model;

/// The rate the model listens at.
const RATE: usize = 16_000;
/// The shortest sound the model is asked about: half a second.
pub const MIN_SAMPLES: usize = RATE / 2;
/// The longest part the model hears at once: 10 seconds. A longer stretch is cut in equal parts.
const MAX_PART: usize = 10 * RATE;

/// The emotion a stretch of speech is said with: the likeliest one, its probability, and the
/// probability of every emotion.
#[derive(Debug, Clone, PartialEq)]
pub struct Emotion {
    /// The likeliest emotion (one of `EMOTIONS`).
    pub label: &'static str,
    /// Its probability, from 0 to 1.
    pub confidence: f32,
    /// The probability of each of `EMOTIONS`, in that order.
    pub scores: Vec<f32>,
}

/// `samples` scaled to zero mean and unit variance, as the model was trained.
fn normalised(samples: &[f32]) -> Vec<f32> {
    let count = samples.len() as f32;
    let mean = samples.iter().sum::<f32>() / count;
    let variance = samples.iter().map(|s| (s - mean) * (s - mean)).sum::<f32>() / count;
    let scale = 1.0 / (variance + 1e-7).sqrt();
    samples.iter().map(|s| (s - mean) * scale).collect()
}

/// The probabilities of the scores (softmax).
fn probabilities(scores: &[f32]) -> Vec<f32> {
    let top = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let exponents: Vec<f32> = scores.iter().map(|score| (score - top).exp()).collect();
    let total: f32 = exponents.iter().sum();
    exponents.iter().map(|value| value / total).collect()
}

/// The pieces a stretch of `length` samples is heard in: as many equal parts as keep each at most
/// 10 seconds, as start and end.
pub fn parts(length: usize) -> Vec<(usize, usize)> {
    let count = length.div_ceil(MAX_PART).max(1);
    (0..count).map(|n| (n * length / count, (n + 1) * length / count)).collect()
}

/// The emotion `samples` (mono, 16 kHz, one stretch of speech) are said with. A stretch longer
/// than 10 seconds is heard in parts and their probabilities are averaged, each weighed by its
/// length. `None` when it is shorter than half a second.
pub fn classify_emotion(file: &Path, device: DeviceChoice, samples: &[f32]) -> Result<Option<Emotion>, Failure> {
    if samples.len() < MIN_SAMPLES {
        return Ok(None);
    }
    let mut sum = vec![0.0f32; EMOTIONS.len()];
    for (start, end) in parts(samples.len()) {
        let scores = run_emotion_model(file, device, &normalised(&samples[start..end]))?;
        for (total, probability) in sum.iter_mut().zip(probabilities(&scores)) {
            *total += probability * (end - start) as f32;
        }
    }
    let scores: Vec<f32> = sum.iter().map(|total| total / samples.len() as f32).collect();
    let (best, confidence) = scores
        .iter()
        .copied()
        .enumerate()
        .fold((0, f32::NEG_INFINITY), |best, (index, value)| if value > best.1 { (index, value) } else { best });
    Ok(Some(Emotion { label: EMOTIONS[best], confidence, scores }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_stretch_is_heard_in_equal_parts_of_at_most_ten_seconds() {
        assert_eq!(parts(5 * RATE), vec![(0, 5 * RATE)]);
        assert_eq!(parts(10 * RATE), vec![(0, 10 * RATE)]);
        assert_eq!(parts(25 * RATE), vec![(0, 25 * RATE / 3), (25 * RATE / 3, 50 * RATE / 3), (50 * RATE / 3, 25 * RATE)]);
    }

    #[test]
    fn the_sound_is_scaled_to_zero_mean_and_unit_variance() {
        let scaled = normalised(&[1.0, 3.0, 1.0, 3.0]);
        assert!(scaled.iter().zip([-1.0, 1.0, -1.0, 1.0]).all(|(a, b)| (a - b).abs() < 1e-3), "{scaled:?}");
    }

    #[test]
    fn probabilities_add_up_to_one() {
        let probabilities = probabilities(&[1.0, 2.0, 3.0]);
        assert!((probabilities.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        assert!(probabilities[2] > probabilities[1]);
    }

    #[test]
    fn a_stretch_under_half_a_second_has_no_emotion_and_needs_no_model() {
        let emotion = classify_emotion(Path::new("no-model"), DeviceChoice::Cpu, &[0.1; MIN_SAMPLES - 1]).unwrap();
        assert!(emotion.is_none());
    }
}
