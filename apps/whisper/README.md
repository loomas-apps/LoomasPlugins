# whisper

Writes down **what is said** in a video or a sound, with the time each sentence starts and ends, using [whisper.cpp](https://github.com/ggml-org/whisper.cpp). It does the same job as [speech-to-text](../speech-to-text) and takes and gives the same things, so a flow can use either. The difference is where it runs: this plugin uses a **graphics card through Vulkan**, from AMD, NVIDIA or Intel, and the processor when there is no card. speech-to-text uses an NVIDIA card or the processor only. A Loomas plugin written in Rust: one program, no Python.

## `transcribe`

| Input | Read |
|---|---|
| `media` | the video or the sound (Loomas decodes it at 16 kHz for the plugin) |
| `model` | `tiny`, `base`, `small`, `medium` (each also as `.en`, English only), `large-v3` or `large-v3-turbo` (default `base`). Downloaded once, on first use |
| `language` | the language spoken, as a code (`pt`, `en`...); found out from the sound when it is left out |
| `words` | give each sentence its words, with their times and confidence (default false) |
| `skipSilence` | leave out the stretches the model thinks are most likely silence (default false) |
| `beamSize` | how many alternatives are weighed, 1 to 10 (default 5) |
| `device` | `auto` (what Loomas grants, else the first card), `cpu` or `gpu` |

It gives `{ language, durationMs, model, text, sentences, device }`. A sentence is `{ startMs, endMs, text, confidence }`, with `words` when asked. Its end is always after its start, so the sentences can go straight to [voice-tone](../voice-tone). `device` says what ran: `cpu` or `vulkan:N`, and in `fallback` why a card asked for was not used.

```json
{ "Words": { "type": "whisper.transcribe",
             "inputs": { "media": "@body('Open')", "model": "large-v3-turbo", "language": "pt", "device": "gpu" } } }
```

## Which model

Measured on 2026-11-11, on a video of 3 min 44 s in Portuguese, with people talking over music, on an AMD Radeon RX 7900 XT and a 16-thread processor. The times include loading the model:

| Model | Where | Time | What it heard |
|---|---|---|---|
| `base` | the card | 4.4 s | music only (`[Música]`) |
| `large-v3-turbo` | the card | 9.8 s | the speech, correctly, with a few repeated shouts |
| `base` | the processor | 14.9 s | music only |
| `base.en` in speech-to-text | the processor | 22 s | English words that were not said |

On 36 s of clear English speech with `base.en` on the processor, it takes 3.5 s, and speech-to-text 7.4 s.

So, for a language other than English or for speech over noise, use `large-v3-turbo` on a card (1.6 GB, downloaded once). The small models are fast but miss speech under music. Always give `language` when you know it.

## Building

The processor build is a normal `cargo build --release -p loomasplugins-whisper`. It needs CMake, a C++ compiler and libclang (on Windows, LLVM: `winget install LLVM.LLVM`, then `LIBCLANG_PATH` set to its `bin` folder). On Windows, `.cargo/config.toml` gives CMake the toolchain file `tools/whisper-msvc.cmake`: without it, the `cmake` crate compiles whisper.cpp with no optimisation, and it runs about ten times slower.

The graphics card build needs the Vulkan SDK too (`winget install KhronosGroup.VulkanSDK`). Make it with:

```sh
sh tools/build-whisper.sh
```

On Windows, the folders whisper.cpp makes inside `dist/cargo` pass the 260-character limit of the compiler, so the script builds in a short folder (`C:/lw`, or `WHISPER_TARGET_DIR`). It then copies the program to `dist/cargo/release`, where `plugin.json` starts it. `sh tools/build-whisper.sh --cpu` builds without Vulkan, in `dist/cargo`.

## What to expect

- **Repetition.** Whisper can repeat a word or a shout many times in a long, noisy stretch. Read the confidence of each sentence.
- **Threads.** On the processor it uses one thread for each physical core (half the logical processors, at most 8). With every logical processor it was four times slower.
- **The card's name** is not given yet (`device.name` is null). whisper.cpp writes it in its log.
- **Tested** only on Windows, with an AMD card. Not tried on Linux, on NVIDIA or Intel cards, or on macOS.

## Failures

`400` a param is wrong, or media is not sound decoded at 16 kHz. `422` the model cannot be downloaded or kept. `500` whisper.cpp could not load the model or failed. `CANCELLED` the call was cancelled. [plugin.json](plugin.json) lists the schemas.
