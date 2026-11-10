use std::path::Path;

use loomasplugins_onnx_device::DeviceChoice;
use loomasplugins_plugin_sdk::Failure;

use crate::models::{run_model, Model};

use super::decode_faces_algorithm::{decode_faces, non_max_suppression};
use super::face_contract::Face;
use super::prepare_detector_input_algorithm::{prepare_detector_input, SIDE};

/// Two faces whose boxes share more than this are one face (as in OpenCV's demo).
const MAX_OVERLAP: f32 = 0.3;

/// The faces in a colour picture (`rgb`, `width` by `height`) the detector is at least
/// `min_score` sure of, the surest first, in the picture's pixels.
pub fn detect_faces(
    file: &Path,
    device: DeviceChoice,
    rgb: &[u8],
    width: usize,
    height: usize,
    min_score: f32,
) -> Result<Vec<Face>, Failure> {
    let (input, scale) = prepare_detector_input(rgb, width, height);
    let outputs = run_model(Model::Faces, file, device, vec![1, 3, SIDE, SIDE], input)?;
    let faces = decode_faces(&outputs, min_score)
        .ok_or_else(|| Failure::status(422, "the face model gave an answer of a shape this plugin does not know"))?;
    Ok(non_max_suppression(faces, MAX_OVERLAP)
        .into_iter()
        .map(|face| Face {
            x: face.x / scale,
            y: face.y / scale,
            width: face.width / scale,
            height: face.height / scale,
            points: face.points.map(|(x, y)| (x / scale, y / scale)),
            ..face
        })
        .collect())
}
