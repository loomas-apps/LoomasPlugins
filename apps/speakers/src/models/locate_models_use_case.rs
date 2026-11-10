use std::path::PathBuf;

use loomasplugins_model_cache::locate_model;
use loomasplugins_plugin_sdk::{Cancel, Failure};

use super::model_catalog_config::{SEGMENTATION, VOICE_PRINT};

/// Where the two models are on this computer.
pub struct ModelPaths {
    /// The model that says who speaks when.
    pub segmentation: PathBuf,
    /// The model that makes a voice print.
    pub voice_print: PathBuf,
}

/// The files of both models, downloaded first when they are not on the computer yet (see the
/// model cache).
pub fn locate_models(cancel: &Cancel) -> Result<ModelPaths, Failure> {
    let get = |file| {
        locate_model("speakers", file, |message| cancel.progress(message))
            .map_err(|error| Failure::status(error.code, error.message).retryable(error.retryable))
    };
    Ok(ModelPaths {
        segmentation: get(&SEGMENTATION)?,
        voice_print: get(&VOICE_PRINT)?,
    })
}
