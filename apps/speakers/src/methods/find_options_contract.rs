use loomasplugins_onnx_device::DeviceChoice;

/// The settings of a `find` call, read and checked.
#[derive(Debug, Clone, PartialEq)]
pub struct FindOptions {
    /// The most voices to report; `None` lets the clustering decide.
    pub max_speakers: Option<usize>,
    /// How similar a voice must be to a pattern to take its name.
    pub min_similarity: f32,
    /// Whether a pattern learns from a voice it matched.
    pub learn: bool,
    /// How similar a voice must be to a pattern for the pattern to learn from it.
    pub learn_similarity: f32,
    /// Whether to write `voice.json` and `sample.wav` for each voice.
    pub files: bool,
    /// The longest sample of a voice, in milliseconds.
    pub sample_ms: u64,
    /// Where the models run.
    pub device: DeviceChoice,
}
