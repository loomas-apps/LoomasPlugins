use std::path::Path;

use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::Failure;

use crate::detect::Face;
use crate::models::{run_model, Model};

use super::align_face_algorithm::{align_face, FACE_SIDE};

/// The expressions the model scores, in the order of its output.
pub const EXPRESSIONS: [&str; 7] = ["angry", "disgust", "fear", "happy", "neutral", "sad", "surprise"];

/// The expression a face shows: the likeliest one and its probability.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Expression {
    /// The place of the likeliest expression in `EXPRESSIONS`.
    pub label: usize,
    /// Its probability, from 0 to 1.
    pub confidence: f32,
}

/// The expression of `face` in a colour picture (`rgb`, `width` by `height`): the face aligned on
/// its five points and given to the expression model, its scores turned into probabilities.
pub fn classify_expression(
    file: &Path,
    device: DeviceChoice,
    rgb: &[u8],
    width: usize,
    height: usize,
    face: &Face,
) -> Result<Expression, Failure> {
    let input = align_face(rgb, width, height, &face.points);
    let outputs = run_model(Model::Expressions, file, device, vec![1, 3, FACE_SIDE, FACE_SIDE], input)?;
    let scores = outputs
        .first()
        .map(|(_, scores)| scores.as_slice())
        .filter(|scores| scores.len() == EXPRESSIONS.len())
        .ok_or_else(|| Failure::status(422, format!("the expression model did not give {} scores", EXPRESSIONS.len())))?;
    let top = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let total: f32 = scores.iter().map(|score| (score - top).exp()).sum();
    let label = scores.iter().position(|score| *score == top).unwrap_or(0);
    Ok(Expression { label, confidence: 1.0 / total })
}
