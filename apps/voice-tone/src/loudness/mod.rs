//! How loud speech is, against the usual speech of the same video. Signal processing only.

mod loudness_class_policy;
mod speech_level_algorithm;

pub use loudness_class_policy::{loudness_class, reference_level};
pub use speech_level_algorithm::speech_level;
