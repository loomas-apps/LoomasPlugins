use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use ort::environment::Environment;
use ort::session::Session;
use ort::value::Tensor;
use loomasplugins_onnx_device::{open_session, DeviceChoice, DeviceReport};
use loomasplugins_plugin_sdk::Failure;

use super::emotion_labels_config::EMOTIONS;

/// The loaded model, the device it was asked to run on and what it really runs on.
struct Loaded {
    asked: DeviceChoice,
    session: Session,
    report: DeviceReport,
}

/// The loaded model, kept until the plugin stops: loading takes far longer than a sentence. It is
/// loaded again when a call asks for another device.
fn session() -> MutexGuard<'static, Option<Loaded>> {
    static SESSION: OnceLock<Mutex<Option<Loaded>>> = OnceLock::new();
    SESSION
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What the model really runs on, once it is loaded.
pub fn device_report() -> Option<DeviceReport> {
    session().as_ref().map(|loaded| loaded.report.clone())
}

fn environment() -> Result<&'static Environment, Failure> {
    static ENVIRONMENT: OnceLock<Result<Environment, String>> = OnceLock::new();
    ENVIRONMENT
        .get_or_init(|| {
            ort::init()
                .with_telemetry(false)
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|message| Failure::status(500, format!("ONNX Runtime did not start: {message}")))
}

/// Gives the model `samples` (mono, 16 kHz, already scaled to zero mean and unit variance) and
/// returns its score for each of the `EMOTIONS`. `file` is the model, loaded on the first call
/// (or when `device` is not the one it was loaded for).
pub fn run_emotion_model(file: &Path, device: DeviceChoice, samples: &[f32]) -> Result<Vec<f32>, Failure> {
    let broken =
        |what: &str, error: &dyn std::fmt::Display| Failure::status(500, format!("the emotion model: {what}: {error}"));
    let mut loaded = session();
    if loaded.as_ref().is_none_or(|current| current.asked != device) {
        let (session, report) = open_session(environment()?, device, file).map_err(|error| {
            Failure::status(
                422,
                format!("the emotion model cannot be loaded from {}: {error}", file.display()),
            )
        })?;
        *loaded = Some(Loaded { asked: device, session, report });
    }
    let session = &mut loaded
        .as_mut()
        .ok_or_else(|| Failure::status(500, "the emotion model is not loaded"))?
        .session;
    let input_name = session
        .inputs()
        .first()
        .map(|input| input.name().to_string())
        .ok_or_else(|| Failure::status(422, "the emotion model has no input"))?;
    let input = Tensor::from_array(([1, samples.len()], samples.to_vec()))
        .map_err(|error| broken("the sound could not be given", &error))?;
    let outputs = session
        .run(ort::inputs![input_name.as_str() => input])
        .map_err(|error| broken("it could not listen", &error))?;
    let (_, scores) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|error| broken("its answer is not numbers", &error))?;
    if scores.len() != EMOTIONS.len() {
        return Err(Failure::status(
            422,
            format!("the emotion model gave {} scores, not {}", scores.len(), EMOTIONS.len()),
        ));
    }
    Ok(scores.to_vec())
}
