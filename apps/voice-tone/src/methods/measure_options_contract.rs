use loomasplugins_onnx_device::DeviceChoice;
use serde_json::Value;

/// The settings of a `measure` call, read and checked.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasureOptions {
    /// The stretches to measure as they were given (each an object with `startMs` and `endMs`),
    /// or `None` to measure the whole sound in windows.
    pub segments: Option<Vec<Value>>,
    /// How long a window is when no segments are given, in milliseconds.
    pub window_ms: u64,
    /// How many dB above or below the usual speech counts as loud or low.
    pub loud_db: f64,
    /// Whether to tell the emotion (which needs the model).
    pub emotions: bool,
    /// Where the model runs.
    pub device: DeviceChoice,
}
