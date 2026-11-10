use realfft::RealFftPlanner;

/// How many filter-bank features a frame has.
pub const BANDS: usize = 80;
/// Samples in a frame: 25 ms at 16 kHz.
const FRAME: usize = 400;
/// Samples between two frames: 10 ms at 16 kHz.
const SHIFT: usize = 160;
/// The frame padded to a power of two for the transform.
const PADDED: usize = 512;
const RATE: f32 = 16_000.0;
const LOW_HZ: f32 = 20.0;
const PRE_EMPHASIS: f32 = 0.97;

fn mel(hz: f32) -> f32 {
    1127.0 * (1.0 + hz / 700.0).ln()
}

/// The weights of each mel band over the first `PADDED / 2` bins of the transform, as Kaldi makes
/// them: triangles evenly spaced on the mel scale from 20 Hz to half the rate.
fn mel_banks() -> Vec<[f32; PADDED / 2]> {
    let low = mel(LOW_HZ);
    let high = mel(RATE / 2.0);
    let delta = (high - low) / (BANDS as f32 + 1.0);
    let bin_hz = RATE / PADDED as f32;
    (0..BANDS)
        .map(|band| {
            let left = low + band as f32 * delta;
            let centre = left + delta;
            let right = centre + delta;
            let mut weights = [0.0f32; PADDED / 2];
            for (bin, weight) in weights.iter_mut().enumerate() {
                let at = mel(bin as f32 * bin_hz);
                let up = (at - left) / (centre - left);
                let down = (right - at) / (right - centre);
                *weight = up.min(down).max(0.0);
            }
            weights
        })
        .collect()
}

/// The log mel energies of each frame, before the mean is taken off.
fn raw_filter_bank(samples: &[f32]) -> Vec<[f32; BANDS]> {
    if samples.len() < FRAME {
        return Vec::new();
    }
    let count = 1 + (samples.len() - FRAME) / SHIFT;
    let window: Vec<f32> = (0..FRAME)
        .map(|n| {
            (0.5 - 0.5 * (2.0 * std::f32::consts::PI * n as f32 / (FRAME as f32 - 1.0)).cos())
                .powf(0.85)
        })
        .collect();
    let banks = mel_banks();
    let fft = RealFftPlanner::<f32>::new().plan_fft_forward(PADDED);
    let mut input = fft.make_input_vec();
    let mut spectrum = fft.make_output_vec();
    let mut power = [0.0f32; PADDED / 2];
    let mut values = [0.0f32; FRAME];
    let mut features = Vec::with_capacity(count);
    for frame in 0..count {
        let part = &samples[frame * SHIFT..frame * SHIFT + FRAME];
        for (value, sample) in values.iter_mut().zip(part) {
            *value = sample * 32_768.0;
        }
        let mean = values.iter().sum::<f32>() / FRAME as f32;
        for value in &mut values {
            *value -= mean;
        }
        input.fill(0.0);
        input[0] = values[0] * (1.0 - PRE_EMPHASIS) * window[0];
        for n in 1..FRAME {
            input[n] = (values[n] - PRE_EMPHASIS * values[n - 1]) * window[n];
        }
        fft.process(&mut input, &mut spectrum)
            .expect("the transform has the sizes it was planned for");
        for (bin, value) in power.iter_mut().enumerate() {
            *value = spectrum[bin].norm_sqr();
        }
        let mut bands = [0.0f32; BANDS];
        for (band, weights) in bands.iter_mut().zip(&banks) {
            let energy: f32 = weights.iter().zip(&power).map(|(w, p)| w * p).sum();
            *band = energy.max(f32::EPSILON).ln();
        }
        features.push(bands);
    }
    features
}

/// The 80 log mel filter-bank features of each 25 ms frame of `samples` (mono, 16 kHz, from -1
/// to 1), every 10 ms, as Kaldi and torchaudio compute them for WeSpeaker: the sound scaled to
/// 16-bit numbers, no dither, the DC taken off, pre-emphasis 0.97, a Povey window, the power
/// spectrum, the log of each band. Then the mean of each feature over the frames is taken off.
/// Fewer than 400 samples give no frame.
pub fn filter_bank(samples: &[f32]) -> Vec<[f32; BANDS]> {
    let mut features = raw_filter_bank(samples);
    if features.is_empty() {
        return features;
    }
    let count = features.len() as f32;
    for band in 0..BANDS {
        let mean = features.iter().map(|frame| frame[band]).sum::<f32>() / count;
        for frame in &mut features {
            frame[band] -= mean;
        }
    }
    features
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Half a second of 440 Hz at half the range with a little 3 kHz, computed in 64 bits as the
    /// reference was (32-bit sines of large angles add noise the quiet bands show).
    fn tone() -> Vec<f32> {
        (0..8000)
            .map(|n| {
                let t = f64::from(n) / 16_000.0;
                (0.5 * (2.0 * std::f64::consts::PI * 440.0 * t).sin()
                    + 0.1 * (2.0 * std::f64::consts::PI * 3000.0 * t).sin()) as f32
            })
            .collect()
    }

    #[test]
    fn the_raw_features_are_those_of_torchaudio() {
        // torchaudio.compliance.kaldi.fbank(tone * 32768, num_mel_bins=80, dither=0,
        // energy_floor=0). The top band of this tone holds almost no energy (its log is near 0
        // against peaks near 16), so its value is rounding noise in both and is not compared.
        let expected = [
            (0, 0, 9.233_793),
            (0, 10, 16.220_88),
            (0, 40, 4.780_804),
            (20, 5, 11.689_653),
            (20, 60, 6.739_081),
        ];
        let raw = raw_filter_bank(&tone());
        assert_eq!(raw.len(), 48);
        for (frame, band, value) in expected {
            assert!(
                (raw[frame][band] - value).abs() < 0.02,
                "frame {frame} band {band}: {} not {value}",
                raw[frame][band]
            );
        }
    }

    #[test]
    fn the_mean_of_each_feature_is_taken_off() {
        let features = filter_bank(&tone());
        for band in [0, 10, 79] {
            let mean: f32 = features.iter().map(|frame| frame[band]).sum::<f32>() / 48.0;
            assert!(mean.abs() < 1e-3, "band {band} has mean {mean}");
        }
    }

    #[test]
    fn a_sound_shorter_than_a_frame_has_no_features() {
        assert!(filter_bank(&[0.1; 399]).is_empty());
        assert_eq!(filter_bank(&[0.1; 400]).len(), 1);
    }
}
