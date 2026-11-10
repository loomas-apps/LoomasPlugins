//! The speech models: which ones there are, where each is downloaded from, and the one loaded.

mod model_catalog_config;
mod whisper_context_store;

pub use model_catalog_config::{model_named, model_names};
pub use whisper_context_store::whisper_context;
