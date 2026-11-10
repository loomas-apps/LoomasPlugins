//! The emotion a stretch of speech is said with, from a wav2vec2 classifier.

mod classify_emotion_use_case;
mod emotion_labels_config;
mod emotion_store;

pub use classify_emotion_use_case::classify_emotion;
pub use emotion_labels_config::EMOTIONS;
pub use emotion_store::device_report;
