use std::path::Path;

use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::{Cancel, Failure};

use crate::cluster::{frame_middle, Piece};
use crate::segment::{Window, FRAME_STEP, RATE};
use crate::voiceprint::make_voice_print;

/// The least speech a run needs for a print, in samples (one second). A print of less is not
/// reliable: a 0.7 s "Thank you" made a voice of its own in a test.
const MIN_PIECE: usize = RATE as usize;
/// A pause longer than this, in frames (about half a second), ends a run of a local speaker.
const RUN_GAP: usize = 30;

/// Where a piece comes from: a run of frames of one local speaker of one window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Origin {
    /// The window.
    pub window: usize,
    /// The local speaker of the window.
    pub local: usize,
    /// The first frame of the run.
    pub first: usize,
    /// The last frame of the run.
    pub last: usize,
}

/// The prints of the runs of every local speaker of every window, with where each one comes from.
pub struct Printed {
    /// The pieces, for the clustering.
    pub pieces: Vec<Piece>,
    /// For each piece, where it comes from.
    pub origins: Vec<Origin>,
    /// For each piece, the seconds of speech its print was made from.
    pub seconds: Vec<f32>,
}

/// The runs of frames where local speaker `local` of `window` talks: stretches with no pause
/// longer than `RUN_GAP` frames, as first and last frame.
fn runs_of(window: &Window, local: usize) -> Vec<(usize, usize)> {
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for (frame, label) in window.frames.iter().enumerate() {
        if !label.speakers[local] {
            continue;
        }
        match runs.last_mut() {
            Some(run) if frame - run.1 <= RUN_GAP => run.1 = frame,
            _ => runs.push((frame, frame)),
        }
    }
    runs
}

/// The samples of the frames `first..=last` of `window` where `wanted` says yes, joined in order.
fn speech_of(
    window: &Window,
    samples: &[f32],
    (first, last): (usize, usize),
    wanted: impl Fn(&[bool; 3]) -> bool,
) -> Vec<f32> {
    let mut speech = Vec::new();
    for frame in first..=last {
        if !wanted(&window.frames[frame].speakers) {
            continue;
        }
        let middle = frame_middle(window.start, frame);
        let from = middle.saturating_sub(FRAME_STEP / 2).min(samples.len());
        let to = (middle + FRAME_STEP / 2).min(samples.len());
        speech.extend_from_slice(&samples[from..to]);
    }
    speech
}

/// Makes a print of each run of each local speaker of each window, from the frames where it talks
/// alone, or, when that is under a second, from all its frames. A run with less than a second of
/// speech gets no print (its frames take the voice of the nearest run, see `frame_voices`).
pub fn print_pieces(
    file: &Path,
    device: DeviceChoice,
    windows: &[Window],
    samples: &[f32],
    cancel: &Cancel,
) -> Result<Printed, Failure> {
    let mut printed = Printed { pieces: Vec::new(), origins: Vec::new(), seconds: Vec::new() };
    for (index, window) in windows.iter().enumerate() {
        if cancel.is_cancelled() {
            return Err(Failure::named("CANCELLED", "cancelled"));
        }
        cancel.progress(&format!("making voice prints: window {} of {}", index + 1, windows.len()));
        for local in 0..3 {
            for run in runs_of(window, local) {
                let alone = speech_of(window, samples, run, |talks| {
                    talks[local] && talks.iter().filter(|talking| **talking).count() == 1
                });
                let speech = if alone.len() >= MIN_PIECE {
                    alone
                } else {
                    speech_of(window, samples, run, |talks| talks[local])
                };
                if speech.len() < MIN_PIECE {
                    continue;
                }
                if let Some(print) = make_voice_print(file, device, &speech)? {
                    printed.pieces.push(Piece { window: index, local, print });
                    printed.origins.push(Origin { window: index, local, first: run.0, last: run.1 });
                    printed.seconds.push(speech.len() as f32 / RATE as f32);
                }
            }
        }
    }
    Ok(printed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment::FrameLabel;

    #[test]
    fn a_long_pause_splits_the_runs_of_a_local_speaker_and_a_short_one_does_not() {
        let talks = |frame: usize| (10..50).contains(&frame) || (60..70).contains(&frame) || (120..130).contains(&frame);
        let window = Window {
            start: 0,
            frames: (0..200)
                .map(|frame| FrameLabel { speakers: [talks(frame), false, false], confidence: 0.9 })
                .collect(),
        };
        assert_eq!(runs_of(&window, 0), vec![(10, 69), (120, 129)]);
        assert!(runs_of(&window, 1).is_empty());
    }
}
