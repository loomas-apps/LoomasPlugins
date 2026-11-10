use std::path::Path;

use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{Cancel, Failure};

use crate::models::{run_model, Model};

use super::powerset_mapper::{label_frame, FrameLabel};

/// The rate the models listen at.
pub const RATE: u32 = 16_000;
/// How long a window of the speech model is: 10 seconds.
pub const WINDOW: usize = 160_000;
/// How far apart two windows start: 5 seconds, so each overlaps the next by half.
pub const STEP: usize = 80_000;
/// How far apart two frames of a window start, in samples (about 17 ms).
pub const FRAME_STEP: usize = 270;
/// How many samples a frame looks at; its middle is half of this after its start.
pub const FRAME_SPAN: usize = 990;

/// What the speech model says of one window: where it starts and a label for each frame.
pub struct Window {
    /// The first sample of the window in the sound.
    pub start: usize,
    /// The frames, `FRAME_STEP` samples apart.
    pub frames: Vec<FrameLabel>,
}

/// Where the windows over `length` samples start: every `STEP`, until one reaches the end.
pub fn window_starts(length: usize) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut start = 0;
    while start < length || starts.is_empty() {
        starts.push(start);
        if start + WINDOW >= length {
            break;
        }
        start += STEP;
    }
    starts
}

/// Runs the speech model over `samples` (mono, 16 kHz) in windows of 10 seconds, the last one
/// filled with silence, and labels every frame of each.
pub fn segment_windows(
    file: &Path,
    device: DeviceChoice,
    samples: &[f32],
    cancel: &Cancel,
) -> Result<Vec<Window>, Failure> {
    let starts = window_starts(samples.len());
    let mut windows = Vec::with_capacity(starts.len());
    for (number, &start) in starts.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err(Failure::named("CANCELLED", "cancelled"));
        }
        cancel.progress(&format!("listening for voices: window {} of {}", number + 1, starts.len()));
        let mut window = vec![0.0f32; WINDOW];
        let end = (start + WINDOW).min(samples.len());
        window[..end - start].copy_from_slice(&samples[start..end]);
        let (shape, scores) = run_model(Model::Segmentation, file, device, vec![1, 1, WINDOW], window)?;
        let classes = shape.last().copied().unwrap_or(0);
        if classes != 7 || scores.len() % 7 != 0 {
            return Err(Failure::status(
                422,
                format!("the speech model gave an answer of shape {shape:?}, not frames of 7 classes"),
            ));
        }
        windows.push(Window {
            start,
            frames: scores.chunks(7).map(label_frame).collect(),
        });
    }
    Ok(windows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_overlap_by_half_and_the_last_reaches_the_end() {
        assert_eq!(window_starts(0), vec![0]);
        assert_eq!(window_starts(WINDOW), vec![0]);
        assert_eq!(window_starts(WINDOW + 1), vec![0, STEP]);
        assert_eq!(window_starts(36 * 16_000), (0..7).map(|n| n * STEP).collect::<Vec<_>>());
    }
}
