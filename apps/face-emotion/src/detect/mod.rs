//! Finding the faces in a picture with YuNet: their boxes and five points each.

mod decode_faces_algorithm;
mod detect_faces_use_case;
mod face_contract;
mod prepare_detector_input_algorithm;

pub use detect_faces_use_case::detect_faces;
pub use face_contract::Face;
