# speakers

Finds who speaks when in a video, and which voice each one is, with **no language model**. A Loomas plugin written in Rust: one program, nothing to install. Loomas decodes the sound (16 kHz, one channel) and the plugin only listens to it.

Each voice gets an id (`voice-1`, `voice-2`...), a **pattern** (`voice.json`) and a **sample** to listen to. Type a name in a pattern and give the pattern back the next time: the turns of that voice then carry the name, and the pattern learns from the new speech.

## `find`

| Input | |
|---|---|
| `media` | the video or sound: a stored file, a video as `media.open` gives it, or a path in text |
| `voices` | patterns from earlier runs, up to 100: the folder of a voice (with its `voice.json`), the `voice.json` itself, or its content |
| `maxSpeakers` | the most voices to report, 0 to 50; 0 lets the plugin decide (default 0) |
| `minSimilarity` | how similar a voice must be to a pattern to take its name, 0.1 to 1 (default 0.5) |
| `learn` | let a matched pattern learn from the new speech (default true) |
| `learnSimilarity` | how similar a voice must be to its pattern for the pattern to learn; at least `minSimilarity` (default 0.65) |
| `files` | write `voice.json` and a sample WAV for each voice (default true) |
| `sampleMs` | the longest sample, 2000 to 30000 (default 8000) |
| `device` | `cpu` (default), `auto` or `gpu` |

It gives `{ turns, voices, durationMs, device }`.

A **turn** is when a voice speaks. Two voices can speak at once. `label` is the name of a known voice or its id, so the turns show as labels on the timeline:

```json
{ "label": "Ryan", "voice": "voice-3", "startMs": 12553, "endMs": 15709, "confidence": 0.97 }
```

A **voice** is one for each voice heard, in the order they first speak:

```json
{ "voice": "voice-3", "name": "Ryan", "known": true, "similarity": 0.924, "learned": true, "seconds": 6.4,
  "pattern": { "kind": "loomas-voice", "format": 1, "model": "wespeaker-voxceleb-resnet34-LM",
               "id": "voice-3", "name": "Ryan", "seconds": 13.8, "print": [0.0123, "... 256 numbers"] },
  "patternFile": { "blob": "sha256:…", "mime": "application/json", "path": "blobs/….json" },
  "sample": { "blob": "sha256:…", "mime": "audio/wav", "path": "blobs/….wav" } }
```

- `known` says whether the voice matched a pattern given in `voices`, and `similarity` says how closely (cosine similarity of the prints, from -1 to 1).
- `seconds` is the speech of this video. `pattern.seconds` is all the speech the pattern was learned from.
- A new voice gets the first `voice-N` after the highest one given, so ids do not clash with known ones.

## Naming voices

1. Run a flow that keeps a folder for each voice (below). You get `voices/voice-1/voice.json` and `voices/voice-1/sample.wav`, and the same for each voice.
2. Listen to `sample.wav` and type the name in `voice.json` (`"name": "John"`).
3. Give the folders to the next run in `voices`. A voice that matches takes the name. A voice that matches none is a new `voice-N`.
4. Keep the new folders: the pattern of a matched voice has learned from the new video (when `learnSimilarity` was reached). The plugin never writes over the files you gave it.

```json
{ "Who": { "type": "speakers.find", "runAfter": { "Open": ["Succeeded"] },
           "inputs": { "media": "@body('Open')", "voices": "@parameters('voices')" } },
  "Keep": { "type": "Foreach", "foreach": "@body('Who').voices", "runAfter": { "Who": ["Succeeded"] },
            "actions": {
              "Pattern": { "type": "data.saveText", "inputs": { "text": "@string(item().pattern)",
                           "saveAs": "@concat('voices/', item().voice, '/voice.json')" } },
              "Sample": { "type": "media.saveSound", "inputs": { "sound": "@item().sample",
                          "saveAs": "@concat('voices/', item().voice, '/sample.wav')" } } } } }
```

**Guards.** A pattern learns only from a voice at least `learnSimilarity` alike, so one doubtful match does not move John's pattern towards Paul. A pattern weighs at most 600 seconds against new speech, so it still follows a voice that changes. A pattern made by another model is refused with a 400: prints of two models cannot be compared.

**Privacy.** A voice print is biometric data (a special category of personal data under the GDPR). The plugin keeps prints only in its result. They are stored where the flow saves them, and the plugin sends nothing anywhere. Keep patterns only of people who agreed, and delete a person's folder when they ask.

## What to expect

- **Tested on synthetic speech only** (2026-11-10): two conversations of 36 s and 20 s made with four Piper voices (two men, two women), each line a single speaker, half a second apart. Every turn went to the right voice, and on the second conversation the patterns named from the first gave Ryan (similarity 0.92) and Sue (0.79), with the new woman as `voice-4`. Real recordings (noise, people talking over each other, a voice far from the microphone, children) are not tested yet: try your own before relying on it.
- **Short speech.** A stretch under one second gets no print of its own. Its frames take the voice of the nearest longer stretch of the same speaker in that window. So a person who only says "yes" once may be put with someone else, or left out.
- **Speed.** 36 s of sound took 4 s on a laptop processor, including starting the plugin. Most of the time goes on the voice prints (one for each stretch of speech in each window).
- **The first call downloads the models** (6 MB and 27 MB) and keeps them in `~/.loomas/models/speakers` (or in the folder `LOOMAS_MODELS` names), after checking their size and sha256. Without a network and without the files, the call fails with a 503 and can be tried again.

## How it works

1. **Who speaks when, in each window.** The sound is cut into windows of 10 s, 5 s apart. [pyannote segmentation-3.0](https://huggingface.co/pyannote/segmentation-3.0) (as the ONNX file of `onnx-community`) says, for each frame of about 17 ms, which of up to three local speakers of that window talk. The window does not know who they are.
2. **A print for each stretch.** Each run of a local speaker's speech (split at pauses longer than half a second) of at least one second gets a voice print. The print uses the frames where that speaker talks alone when there is enough of them. The print is 256 numbers from [WeSpeaker ResNet34](https://huggingface.co/Wespeaker/wespeaker-voxceleb-resnet34-LM), the model pyannote 3.1 uses, fed 80 Kaldi filter-bank features computed in the plugin (checked against torchaudio).
3. **Group the prints into voices.** Agglomerative clustering: the two groups whose mean prints are most alike join, while their cosine similarity is at least 0.6. Two local speakers of one window never join, because the speech model said they are two people. Prints of one voice measured 0.56 to 0.87 and of two voices at most 0.2. pyannote's own threshold kept one voice in three groups when a voice has only a few stretches.
4. **Turns.** Each window speaks for the middle half of its time. The frames of each voice join into turns when a pause is shorter than 300 ms, and turns shorter than 250 ms are dropped.
5. **Names.** The mean print of each voice is compared with the patterns given. The most alike pairs match first, and each pattern names one voice at most.

## Licences

The program's own licence is the author's to choose (`loomas plugins check` says there is none yet). What it uses: pyannote segmentation-3.0 is MIT (the ONNX conversion says the same). The WeSpeaker ResNet34 model is **CC-BY-4.0**, by the WeSpeaker project (Wang et al., [arXiv:2210.17016](https://arxiv.org/abs/2210.17016)): any distribution must credit it. ONNX Runtime is MIT. The Rust crates are listed in [docs/licences.md](../../docs/licences.md). The models are **not packaged**: they are downloaded.

## Failures

`400` a param is wrong, a pattern is not one (or comes from another model), or `media` is not sound Loomas decoded for the plugin. `404` a pattern file is not there. `422` a file cannot be read, a model cannot be loaded, or Loomas gave the plugin no folder for the voice files. `503` a model could not be downloaded. [plugin.json](plugin.json) lists them with the schemas.
