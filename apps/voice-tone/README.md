# voice-tone

How each stretch of speech in a video is said: **how loud** it is against the usual speech of the same video (`low`, `normal`, `loud`), and **which emotion** the voice has (angry, calm, disgust, fear, happy, neutral, sad, surprise). No language model. A Loomas plugin written in Rust: one program, nothing to install. Loomas decodes the sound (16 kHz, one channel) and the plugin only listens to it.

## `measure`

| Input | |
|---|---|
| `media` | the video or sound: a stored file, a video as `media.open` gives it, or a path in text |
| `segments` | the stretches to measure, each with `startMs` and `endMs`: the sentences of [speech-to-text](../speech-to-text) or the turns of [speakers](../speakers), as they are (their other fields are kept). Leave it out to measure the sound in windows |
| `windowMs` | how long each window is when no segments are given, 1000 to 30000 (default 3000) |
| `loudDb` | how many dB above the usual speech counts as loud, and below as low, 2 to 20 (default 6) |
| `emotions` | tell the emotion (default true); without it only the loudness is measured and no model is downloaded |
| `device` | `cpu` (default), `auto` or `gpu` |

It gives `{ segments, referenceDb, durationMs, device }`. Each segment keeps its own fields and gains the tone:

```json
{ "label": "Alan", "voice": "voice-1", "startMs": 27553, "endMs": 31755,
  "loudnessDb": -7.5, "relativeDb": 7.0, "loudness": "loud",
  "emotion": "neutral", "emotionConfidence": 0.81,
  "emotions": { "angry": 0.0, "calm": 0.05, "disgust": 0.02, "fear": 0.0, "happy": 0.0, "neutral": 0.81, "sad": 0.11, "surprise": 0.0 } }
```

- `loudnessDb` is the level of the speech in dB below full scale. Pauses between words are left out, so a sentence with pauses is not quieter.
- `referenceDb` is the middle level of all the stretches with speech, and `relativeDb` is the difference from it. "Loud" means loud for this video: a whisper film and a shouting match each have their own normal.
- Without `segments`, the windows with no speech are left out. With `segments`, every one comes back, and what cannot be measured is null (silence, or under half a second for the emotion).

```json
{ "Who": { "type": "speakers.find", "runAfter": { "Open": ["Succeeded"] }, "inputs": { "media": "@body('Open')" } },
  "Tone": { "type": "voice-tone.measure", "runAfter": { "Who": ["Succeeded"] },
            "inputs": { "media": "@body('Open')", "segments": "@body('Who').turns" } } }
```

## What to expect

- **Loudness** is plain signal processing. A line made 10 dB louder in a test conversation came out `loud` (+7 dB against the usual level, because the louder line also raises the usual level a little), and every other line came out `normal`.
- **The emotion is not tested on real speech.** The model was trained on acted speech in English and Hindi. On the synthetic Piper voices of the tests it said mostly `neutral`, `sad` for an apology, and `happy` or `fear` for one calm voice. Real voices, other languages, children and noise will be less certain. Measure it on your own videos before relying on it, and read `emotionConfidence` and `emotions` rather than the label alone.
- **Speed.** About 0.25 s for each sentence on a laptop processor. A stretch longer than 10 s is heard in equal parts of at most 10 s, and their probabilities are averaged.
- **The first call downloads the model** (95 MB) and keeps it in `~/.loomas/models/voice-tone` (or in the folder `LOOMAS_MODELS` names), after checking its size and sha256. Without a network and without the file, the call fails with a 503 and can be tried again.

## How it works

- **Loudness.** The stretch is cut into frames of 50 ms. Frames quieter than -50 dB below full scale count as pauses. The level is the mean energy of the other frames, in dB.
- **Emotion.** The stretch is scaled to zero mean and unit variance and given to a wav2vec2 base classifier (`prithivMLmods/Speech-Emotion-Classification`, as the 8-bit ONNX file of `onnx-community`), run by [ONNX Runtime](https://onnxruntime.ai). The plugin turns its 8 scores into probabilities.

## Licences

The program's own licence is the author's to choose (`loomas plugins check` says there is none yet). What it uses: the emotion model is Apache-2.0, fine-tuned from `facebook/wav2vec2-base-960h` (Apache-2.0) on `stapesai/ssi-speech-emotion-recognition` (listed as MIT). Where that dataset's recordings come from, and so what their terms are, is to be confirmed before the plugin is distributed. ONNX Runtime is MIT. The Rust crates are listed in [docs/licences.md](../../docs/licences.md). The model is **not packaged**: it is downloaded.

## Failures

`400` a param is wrong, a segment has no valid `startMs` and `endMs`, or `media` is not sound Loomas decoded for the plugin. `422` a file cannot be read or the model cannot be loaded. `503` the model could not be downloaded. [plugin.json](plugin.json) lists them with the schemas.
