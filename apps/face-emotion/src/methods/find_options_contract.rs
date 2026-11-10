use loomasplugins_onnx_device::DeviceChoice;

/// The settings of a `find` call, read and checked.
#[derive(Debug, Clone, PartialEq)]
pub struct FindOptions {
    /// How sure the detector must be that it is a face.
    pub min_face_score: f32,
    /// Faces narrower than this, in pixels of the decoded picture, are followed but get no
    /// expression.
    pub min_face_width: f32,
    /// How long an expression can stop before it counts as over, in milliseconds.
    pub gap_ms: u64,
    /// In how few pictures an expression is still reported.
    pub min_samples: usize,
    /// Where the models run.
    pub device: DeviceChoice,
}
