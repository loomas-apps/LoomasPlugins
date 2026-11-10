/// What the speech model says of one frame of a window: which of its three local speakers talk,
/// and how sure it is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameLabel {
    /// Whether each of the three local speakers of the window talks in this frame.
    pub speakers: [bool; 3],
    /// The probability of the class it chose, from 0 to 1.
    pub confidence: f32,
}

/// The local speakers of each of the 7 classes: nobody, 1, 2, 3, 1 and 2, 1 and 3, 2 and 3.
const CLASSES: [[bool; 3]; 7] = [
    [false, false, false],
    [true, false, false],
    [false, true, false],
    [false, false, true],
    [true, true, false],
    [true, false, true],
    [false, true, true],
];

/// Turns the 7 scores of a frame (log-probabilities or plain scores) into the class the model
/// thinks most likely, with its probability.
pub fn label_frame(scores: &[f32]) -> FrameLabel {
    let top = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let total: f32 = scores.iter().map(|score| (score - top).exp()).sum();
    let (best, score) = scores
        .iter()
        .copied()
        .enumerate()
        .take(CLASSES.len())
        .fold((0, f32::NEG_INFINITY), |best, (index, score)| {
            if score > best.1 {
                (index, score)
            } else {
                best
            }
        });
    FrameLabel {
        speakers: CLASSES[best],
        confidence: (score - top).exp() / total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_likeliest_class_is_chosen_with_its_probability() {
        let probabilities = [0.0f32, 0.1, 0.2, 0.0, 0.7, 0.0, 0.0];
        let label = label_frame(&probabilities.map(f32::ln));
        assert_eq!(label.speakers, [true, true, false]);
        assert!((label.confidence - 0.7).abs() < 1e-5, "{}", label.confidence);
    }

    #[test]
    fn silence_is_nobody() {
        let label = label_frame(&[5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(label.speakers, [false; 3]);
        assert!(label.confidence > 0.9);
    }
}
