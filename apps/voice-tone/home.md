# Voice tone

How each stretch of speech in a video is said: **how loud** it is against the usual speech of the same video (`low`, `normal`, `loud`), and **which emotion** the voice has (angry, calm, disgust, fear, happy, neutral, sad, surprise). No language model. A Loomas plugin written in Rust: one program, nothing to install. Loomas decodes the sound (16 kHz, one channel) and the plugin only listens to it.

## What to expect

- **Loudness** is plain signal processing. A line made 10 dB louder in a test conversation came out `loud` (+7 dB against the usual level, because the louder line also raises the usual level a little), and every other line came out `normal`.
- **The emotion is not tested on real speech.** The model was trained on acted speech in English and Hindi. On the synthetic Piper voices of the tests it said mostly `neutral`, `sad` for an apology, and `happy` or `fear` for one calm voice. Real voices, other languages, children and noise will be less certain. Measure it on your own videos before relying on it, and read `emotionConfidence` and `emotions` rather than the label alone.
- **Speed.** About 0.25 s for each sentence on a laptop processor. A stretch longer than 10 s is heard in equal parts of at most 10 s, and their probabilities are averaged.
- **The first call downloads the model** (95 MB) and keeps it in `~/.loomas/models/voice-tone` (or in the folder `LOOMAS_MODELS` names), after checking its size and sha256. Without a network and without the file, the call fails with a 503 and can be tried again.

## Licences

The program's own licence is the author's to choose (`loomas plugins check` says there is none yet). What it uses: the emotion model is Apache-2.0, fine-tuned from `facebook/wav2vec2-base-960h` (Apache-2.0) on `stapesai/ssi-speech-emotion-recognition` (listed as MIT). Where that dataset's recordings come from, and so what their terms are, is to be confirmed before the plugin is distributed. ONNX Runtime is MIT. The Rust crates are listed in docs/licences.md. The model is **not packaged**: it is downloaded.
