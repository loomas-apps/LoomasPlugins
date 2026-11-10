//! The expression a face shows: the face aligned on its five points and given to the expression
//! model of the OpenCV model zoo.

mod align_face_algorithm;
mod classify_expression_use_case;

pub use classify_expression_use_case::{classify_expression, EXPRESSIONS};
