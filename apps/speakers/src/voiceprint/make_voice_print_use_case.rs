use std::path::Path;

use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::Failure;

use crate::models::{run_model, Model};

use super::filter_bank_algorithm::{filter_bank, BANDS};

/// How many numbers a voice print has.
pub const PRINT_SIZE: usize = 256;

/// Scales `print` to length 1, so two prints compare by their direction only. A print of zeros
/// stays zeros.
pub fn unit(mut print: Vec<f32>) -> Vec<f32> {
    let length = print.iter().map(|value| value * value).sum::<f32>().sqrt();
    if length > 0.0 {
        for value in &mut print {
            *value /= length;
        }
    }
    print
}

/// The voice print of `samples` (mono, 16 kHz, one voice), of length 1. `None` when the sound is
/// shorter than one frame (25 ms).
pub fn make_voice_print(
    file: &Path,
    device: DeviceChoice,
    samples: &[f32],
) -> Result<Option<Vec<f32>>, Failure> {
    let features = filter_bank(samples);
    if features.is_empty() {
        return Ok(None);
    }
    let frames = features.len();
    let data: Vec<f32> = features.into_iter().flatten().collect();
    let (_, print) = run_model(Model::VoicePrint, file, device, vec![1, frames, BANDS], data)?;
    if print.len() != PRINT_SIZE {
        return Err(Failure::status(
            422,
            format!("the voice print model gave {} numbers, not {PRINT_SIZE}", print.len()),
        ));
    }
    Ok(Some(unit(print)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_unit_print_has_length_one_and_zeros_stay_zeros() {
        let print = unit(vec![3.0, 4.0]);
        assert!((print[0] - 0.6).abs() < 1e-6 && (print[1] - 0.8).abs() < 1e-6);
        assert_eq!(unit(vec![0.0, 0.0]), vec![0.0, 0.0]);
    }

    #[test]
    fn a_sound_too_short_has_no_print_and_needs_no_model() {
        let print = make_voice_print(Path::new("no-model"), DeviceChoice::Cpu, &[0.0; 100]).unwrap();
        assert!(print.is_none());
    }
}
