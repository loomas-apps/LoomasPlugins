use loomasplugins_model_cache::locate_model;
use loomasplugins_plugin_sdk::{Cancel, Failure};
use loomasplugins_sound_reader::read_audio_region;
use serde_json::{json, Value};
use whisper_rs::{FullParams, SamplingStrategy};

use crate::device::plan_device;
use crate::models::{model_named, whisper_context};
use crate::speech::{sentences_of, HeardSegment, HeardToken, Placing};

use super::read_transcribe_options_mapper::read_transcribe_options;

/// The rate whisper.cpp listens at.
const RATE: u32 = 16_000;

/// The processor threads whisper.cpp uses: one for each physical core (half the logical ones), at
/// most 8. With every logical processor it ran four times slower on a 16-thread processor (3.9 s
/// with 8 threads, 4.8 s with 4, 16.5 s with 16, for 36 s of speech).
fn thread_count(logical: usize) -> i32 {
    i32::try_from((logical / 2).clamp(1, 8)).unwrap_or(8)
}

/// Tells whisper.cpp to stop when the call was cancelled; `data` points to the call's `Cancel`.
unsafe extern "C" fn stop_when_cancelled(data: *mut std::ffi::c_void) -> bool {
    // SAFETY: `transcribe` passes a pointer to a `Cancel` that lives while whisper.cpp runs.
    unsafe { (*data.cast::<Cancel>()).is_cancelled() }
}

fn failed(message: impl Into<String>) -> Failure {
    Failure::status(500, message)
}

/// The `transcribe` action: the sentences said in a video or a sound, with the time each starts and
/// ends, in the shape of the speech-to-text plugin.
///
/// Params: `media` (sound Loomas decoded at 16 kHz), and optional `model` (default `base`),
/// `language` (a code; found out when left out), `words`, `skipSilence`, `beamSize` (1 to 10,
/// default 5) and `device` (`auto`, `cpu`, `gpu`, `gpu:N`). It gives `{ language, durationMs,
/// model, text, sentences, device }`. Fails with 400 for a wrong param, with the code of the model
/// cache when the model cannot be had, with 500 when whisper.cpp fails, and with `CANCELLED`.
pub fn transcribe(params: &Value, cancel: &Cancel) -> Result<Value, Failure> {
    let options = read_transcribe_options(params)?;
    let media = params
        .get("media")
        .ok_or_else(|| Failure::status(400, "media is missing: give a video or a sound"))?;
    let sound = read_audio_region(media)
        .map_err(|error| Failure::status(error.code, error.message))?
        .ok_or_else(|| {
            Failure::status(
                400,
                "media must be sound Loomas decoded for this plugin (it asks for 16 kHz sound)",
            )
        })?;
    if sound.sample_rate != RATE {
        return Err(Failure::status(
            400,
            format!(
                "this plugin listens at {RATE} Hz, not {} Hz",
                sound.sample_rate
            ),
        ));
    }
    let plan = plan_device(
        options.device,
        cancel.granted_device(),
        cfg!(feature = "vulkan"),
    );
    let device = json!({
        "device": plan.gpu.map_or_else(|| "cpu".to_string(), |number| format!("vulkan:{number}")),
        "name": null,
        "fallback": plan.fallback,
    });
    let duration_ms = sound.duration_ms();
    let empty = |language: &str| json!({ "language": language, "durationMs": duration_ms, "model": options.model, "text": "", "sentences": [], "device": device });
    if sound.samples.is_empty() {
        return Ok(empty(options.language.as_deref().unwrap_or("en")));
    }

    let file = model_named(&options.model).ok_or_else(|| Failure::status(400, "unknown model"))?;
    let path = locate_model("whisper", file, |message| cancel.progress(message))
        .map_err(|error| Failure::status(error.code, error.message).retryable(error.retryable))?;
    let context = whisper_context(&path, plan.gpu).map_err(failed)?;
    let mut state = context
        .create_state()
        .map_err(|error| failed(format!("whisper.cpp cannot start: {error}")))?;

    let strategy = if options.beam_size > 1 {
        SamplingStrategy::BeamSearch {
            beam_size: i32::try_from(options.beam_size).unwrap_or(5),
            patience: -1.0,
        }
    } else {
        SamplingStrategy::Greedy { best_of: 1 }
    };
    let mut full = FullParams::new(strategy);
    full.set_language(Some(options.language.as_deref().unwrap_or("auto")));
    full.set_token_timestamps(options.words);
    full.set_print_special(false);
    full.set_print_progress(false);
    full.set_print_realtime(false);
    full.set_print_timestamps(false);
    let threads =
        thread_count(std::thread::available_parallelism().map_or(2, std::num::NonZeroUsize::get));
    full.set_n_threads(threads);
    // The safe abort callback of whisper-rs 0.16 crashes when whisper.cpp calls it, so the raw one
    // reads the call's `Cancel` through a pointer; `cancel` outlives `state.full` below.
    // SAFETY: the pointer is to a `Cancel` borrowed for this whole function, and whisper.cpp calls
    // the callback only during `state.full`.
    unsafe {
        full.set_abort_callback(Some(stop_when_cancelled));
        full.set_abort_callback_user_data(std::ptr::from_ref(cancel).cast_mut().cast());
    }
    let progress = cancel.clone();
    full.set_progress_callback_safe(move |percent: i32| {
        progress.progress(&format!("{percent}% heard"))
    });

    let result = state.full(full, &sound.samples);
    if cancel.is_cancelled() {
        return Err(Failure::named("CANCELLED", "cancelled"));
    }
    result.map_err(|error| failed(format!("whisper.cpp failed: {error}")))?;

    let segments: Vec<HeardSegment> = state
        .as_iter()
        .map(|segment| HeardSegment {
            start_cs: segment.start_timestamp(),
            end_cs: segment.end_timestamp(),
            text: segment
                .to_str_lossy()
                .map(|text| text.into_owned())
                .unwrap_or_default(),
            no_speech: segment.no_speech_probability(),
            tokens: (0..segment.n_tokens())
                .filter_map(|index| segment.get_token(index))
                .map(|token| {
                    let data = token.token_data();
                    HeardToken {
                        text: token.to_str().unwrap_or_default().to_string(),
                        probability: data.p,
                        start_cs: data.t0,
                        end_cs: data.t1,
                    }
                })
                .collect(),
        })
        .collect();
    let language = options
        .language
        .clone()
        .or_else(|| whisper_rs::get_lang_str(state.full_lang_id_from_state()).map(str::to_string))
        .unwrap_or_else(|| "en".to_string());
    let placing = Placing {
        offset_ms: sound.start_ms,
        duration_ms,
        words: options.words,
        skip_silence: options.skip_silence,
    };
    let sentences = sentences_of(&segments, &placing);
    let text = sentences
        .iter()
        .filter_map(|sentence| sentence["text"].as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let mut answer = empty(&language);
    answer["text"] = json!(text);
    answer["sentences"] = Value::Array(sentences);
    Ok(answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whisper_uses_one_thread_for_each_physical_core_and_at_most_eight() {
        assert_eq!(
            (
                thread_count(16),
                thread_count(32),
                thread_count(4),
                thread_count(1)
            ),
            (8, 8, 2, 1)
        );
    }
}
