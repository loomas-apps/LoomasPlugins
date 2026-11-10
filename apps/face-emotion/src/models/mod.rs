//! The two models: where they come from, where they are kept, and running them.

mod locate_models_use_case;
mod model_catalog_config;
mod onnx_store;

pub use locate_models_use_case::locate_models;
pub use onnx_store::{device_report, run_model, Model};
