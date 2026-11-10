use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use ort::environment::Environment;
use ort::session::Session;
use ort::value::Tensor;
use loomasplugins_onnx_device::{open_session, DeviceChoice, DeviceReport};
use loomasplugins_plugin_sdk::Failure;

/// The two models this plugin runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    /// The face detector.
    Faces,
    /// The expression model.
    Expressions,
}

impl Model {
    fn slot(self) -> usize {
        self as usize
    }

    fn what(self) -> &'static str {
        match self {
            Model::Faces => "the face model",
            Model::Expressions => "the expression model",
        }
    }
}

/// A loaded model, the device it was asked to run on and what it really runs on.
struct Loaded {
    asked: DeviceChoice,
    session: Session,
    report: DeviceReport,
}

/// The loaded models, kept until the plugin stops: loading takes far longer than a picture. A
/// model is loaded again when a call asks for another device.
fn sessions() -> MutexGuard<'static, [Option<Loaded>; 2]> {
    static SESSIONS: OnceLock<Mutex<[Option<Loaded>; 2]>> = OnceLock::new();
    SESSIONS
        .get_or_init(|| Mutex::new([None, None]))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What the face model really runs on, once it is loaded.
pub fn device_report() -> Option<DeviceReport> {
    sessions()[Model::Faces.slot()].as_ref().map(|loaded| loaded.report.clone())
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

/// Gives `model` (loaded from `file` on the first call, or when `device` changes) one input of
/// `shape` with `data`, and returns every output by name, flattened.
pub fn run_model(
    model: Model,
    file: &Path,
    device: DeviceChoice,
    shape: Vec<usize>,
    data: Vec<f32>,
) -> Result<Vec<(String, Vec<f32>)>, Failure> {
    let broken =
        |what: &str, error: &dyn std::fmt::Display| Failure::status(500, format!("{}: {what}: {error}", model.what()));
    let mut all = sessions();
    let slot = &mut all[model.slot()];
    if slot.as_ref().is_none_or(|current| current.asked != device) {
        let (session, report) = open_session(environment()?, device, file).map_err(|error| {
            Failure::status(422, format!("{} cannot be loaded from {}: {error}", model.what(), file.display()))
        })?;
        *slot = Some(Loaded { asked: device, session, report });
    }
    let session = &mut slot
        .as_mut()
        .ok_or_else(|| Failure::status(500, format!("{} is not loaded", model.what())))?
        .session;
    let input_name = session
        .inputs()
        .first()
        .map(|input| input.name().to_string())
        .ok_or_else(|| Failure::status(422, format!("{} has no input", model.what())))?;
    let output_names: Vec<String> = session.outputs().iter().map(|output| output.name().to_string()).collect();
    let input = Tensor::from_array((shape, data)).map_err(|error| broken("the input could not be given", &error))?;
    let outputs = session
        .run(ort::inputs![input_name.as_str() => input])
        .map_err(|error| broken("it did not run", &error))?;
    output_names
        .into_iter()
        .enumerate()
        .map(|(index, name)| {
            let (_, numbers) = outputs[index]
                .try_extract_tensor::<f32>()
                .map_err(|error| broken("its answer is not numbers", &error))?;
            Ok((name, numbers.to_vec()))
        })
        .collect()
}
