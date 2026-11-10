use crate::device::DeviceChoice;

/// The settings of a `transcribe` call.
#[derive(Debug, Clone, PartialEq)]
pub struct TranscribeOptions {
    /// The model name (`base`, `small.en`...).
    pub model: String,
    /// The language as a code, or `None` to find it out.
    pub language: Option<String>,
    pub words: bool,
    pub skip_silence: bool,
    /// How many alternatives are weighed; 1 takes the likeliest word each time.
    pub beam_size: u64,
    pub device: DeviceChoice,
}
