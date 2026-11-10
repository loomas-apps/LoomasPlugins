use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use whisper_rs::{WhisperContext, WhisperContextParameters};

/// The model loaded last, with the file and the card it was loaded for. Only one is kept, because a
/// large model takes gigabytes and a run uses one model.
static LOADED: OnceLock<Mutex<Option<Loaded>>> = OnceLock::new();

/// A loaded model: its file, its card, and whisper.cpp's context.
type Loaded = (PathBuf, Option<i32>, Arc<WhisperContext>);

/// The whisper.cpp model in `file`, on the graphics card `gpu` (or the processor for `None`),
/// loaded once and kept for the next calls of the run.
///
/// Fails with the reason when whisper.cpp cannot read the file or start on the card.
pub fn whisper_context(file: &Path, gpu: Option<i32>) -> Result<Arc<WhisperContext>, String> {
    let mut loaded = LOADED
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((path, card, context)) = loaded.as_ref() {
        if path == file && *card == gpu {
            return Ok(Arc::clone(context));
        }
    }
    *loaded = None;
    let mut params = WhisperContextParameters::default();
    params.use_gpu(gpu.is_some());
    if let Some(number) = gpu {
        params.gpu_device(number);
    }
    let path = file
        .to_str()
        .ok_or_else(|| format!("the model path {} is not valid text", file.display()))?;
    let context = Arc::new(
        WhisperContext::new_with_params(path, params)
            .map_err(|error| format!("whisper.cpp cannot load {path}: {error}"))?,
    );
    *loaded = Some((file.to_path_buf(), gpu, Arc::clone(&context)));
    Ok(context)
}
