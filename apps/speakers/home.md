# Find speakers

Finds who speaks when in a video, and which voice each one is, with **no language model**. A Loomas plugin written in Rust: one program, nothing to install. Loomas decodes the sound (16 kHz, one channel) and the plugin only listens to it.

Each voice gets an id (`voice-1`, `voice-2`...), a **pattern** (`voice.json`) and a **sample** to listen to. Type a name in a pattern and give the pattern back the next time: the turns of that voice then carry the name, and the pattern learns from the new speech.

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

## Licences

The program's own licence is the author's to choose (`loomas plugins check` says there is none yet). What it uses: pyannote segmentation-3.0 is MIT (the ONNX conversion says the same). The WeSpeaker ResNet34 model is **CC-BY-4.0**, by the WeSpeaker project (Wang et al., [arXiv:2210.17016](https://arxiv.org/abs/2210.17016)): any distribution must credit it. ONNX Runtime is MIT. The Rust crates are listed in docs/licences.md. The models are **not packaged**: they are downloaded.
