use std::path::PathBuf;

use loomasplugins_model_cache::locate_model;
use loomasplugins_plugin_sdk::{Cancel, Failure};

use super::model_catalog_config::{EXPRESSIONS, FACES};

/// Where the two models are on this computer.
pub struct ModelPaths {
    /// The face detector.
    pub faces: PathBuf,
    /// The expression model.
    pub expressions: PathBuf,
}

/// The files of both models, downloaded first when they are not on the computer yet (see the
/// model cache).
pub fn locate_models(cancel: &Cancel) -> Result<ModelPaths, Failure> {
    let get = |file| {
        locate_model("face-emotion", file, |message| cancel.progress(message))
            .map_err(|error| Failure::status(error.code, error.message).retryable(error.retryable))
    };
    Ok(ModelPaths { faces: get(&FACES)?, expressions: get(&EXPRESSIONS)? })
}
