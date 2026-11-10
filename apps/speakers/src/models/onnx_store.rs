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
    /// Who speaks when.
    Segmentation,
    /// The voice print.
    VoicePrint,
}

impl Model {
    fn slot(self) -> usize {
        self as usize
    }

    fn what(self) -> &'static str {
        match self {
            Model::Segmentation => "the speech model",
            Model::VoicePrint => "the voice print model",
        }
    }
}

/// A loaded model, the device it was asked to run on and what it really runs on.
struct Loaded {
    asked: DeviceChoice,
    session: Session,
    report: DeviceReport,
}

/// The loaded models, kept until the plugin stops: loading takes far longer than a window of
/// sound. A model is loaded again when a call asks for another device.
fn sessions() -> MutexGuard<'static, [Option<Loaded>; 2]> {
    static SESSIONS: OnceLock<Mutex<[Option<Loaded>; 2]>> = OnceLock::new();
    SESSIONS
        .get_or_init(|| Mutex::new([None, None]))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What the voice print model really runs on, once it is loaded (the larger of the two).
pub fn device_report() -> Option<DeviceReport> {
    sessions()[Model::VoicePrint.slot()]
        .as_ref()
        .map(|loaded| loaded.report.clone())
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
/// `shape` with `data`, and returns its first output: its shape and its numbers.
pub fn run_model(
    model: Model,
    file: &Path,
    device: DeviceChoice,
    shape: Vec<usize>,
    data: Vec<f32>,
) -> Result<(Vec<usize>, Vec<f32>), Failure> {
    let broken =
        |what: &str, error: &dyn std::fmt::Display| Failure::status(500, format!("{}: {what}: {error}", model.what()));
    let mut all = sessions();
    let slot = &mut all[model.slot()];
    if slot.as_ref().is_none_or(|current| current.asked != device) {
        let (session, report) = open_session(environment()?, device, file).map_err(|error| {
            Failure::status(
                422,
                format!("{} cannot be loaded from {}: {error}", model.what(), file.display()),
            )
        })?;
        *slot = Some(Loaded {
            asked: device,
            session,
            report,
        });
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
    let input = Tensor::from_array((shape, data))
        .map_err(|error| broken("the input could not be given", &error))?;
    let outputs = session
        .run(ort::inputs![input_name.as_str() => input])
        .map_err(|error| broken("it did not run", &error))?;
    let (shape, numbers) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|error| broken("its answer is not numbers", &error))?;
    let shape = shape.iter().map(|&size| size.max(0) as usize).collect();
    Ok((shape, numbers.to_vec()))
}
