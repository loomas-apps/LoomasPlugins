# face-emotion

Finds the **faces** in a video, follows each one from picture to picture, and says which **expression** it shows over time: angry, disgust, fear, happy, neutral, sad or surprise. It **never says who a face is**: faces are numbered `face-1`, `face-2`... in the order they show. No language model. A Loomas plugin written in Rust: one program, nothing to install. Loomas decodes two colour pictures a second, 640 pixels wide, and the plugin only looks at them.

## `find`

| Input | |
|---|---|
| `media` | the video: a video as `media.open` gives it |
| `minFaceScore` | how sure the detector must be that it is a face, 0.3 to 1 (default 0.7) |
| `minFaceWidth` | faces narrower than this, in pixels of the 640-wide picture, are followed but get no expression (default 24) |
| `gapMs` | how long an expression can stop before it counts as over, 0 to 600000 (default 1000) |
| `minSamples` | in how few pictures an expression is still reported, 1 to 100 (default 1) |
| `device` | `auto` (default: what Loomas grants), `cpu` or `gpu` |

It gives `{ expressions, faces, samples, durationMs, device }`. An **expression** is a stretch where one face keeps one expression, so they show as labels on the timeline:

```json
{ "label": "happy", "face": "face-1", "startMs": 0, "endMs": 3000, "confidence": 1.0, "samples": 6,
  "box": { "x": 0.697, "y": 0.373, "width": 0.067, "height": 0.087 } }
```

`box` is where the face was when the stretch began, as shares of the picture's width and height. A **face** is `{ face, startMs, endMs, samples }`: when it first and last shows.

The expression names match those of [voice-tone](../voice-tone) where both have one (the expression model says `fearful` and `surprised`, and this plugin says `fear` and `surprise`). So a narrator can compare a face with its voice.

## What to expect

- **Tested on one picture only** (2026-11-10): a group selfie (OpenCV's YuNet example) made into a 3-second video. The plugin read the same expressions as a Python reference of the same models (`happy`, 1.0 for the three nearest faces), and found 54 faces where OpenCV found 48 in the picture (the video is compressed, and small faces at the edge of the score come and go). Real videos with people who move, turn away and change expression are not tested yet.
- **Faces that turn away** are lost until they turn back. A face not seen for more than 2 seconds that shows again counts as a new face.
- **Small faces.** Under 24 pixels wide (in a 640-wide picture), there are too few pixels to read an expression, so the face is followed but gets none.
- **Speed.** 3 seconds of video took under 3 s, with the start of the plugin, on a graphics card. The detector looks at every picture, and the expression model at every face of at least `minFaceWidth`.
- **The first call downloads the models** (230 kB and 4.8 MB) and keeps them in `~/.loomas/models/face-emotion` (or in the folder `LOOMAS_MODELS` names), after checking their size and sha256.

## Privacy

Faces are personal data. The plugin keeps nothing: no face picture, no face print. It gives only times, boxes and expression names, and it cannot tell who a face is or match a face across videos. An expression is what a face shows, not what a person feels.

## How it works

1. **Faces.** The picture is put in the top left of a 640 by 640 square (made smaller first when a side is longer) and given to [YuNet](https://github.com/ShiqiYu/libfacedetection) (2023). Its answer is decoded as OpenCV's `FaceDetectorYN` does (checked against it: the same 48 boxes). Faces sharing more than 30% of their boxes are one face.
2. **Expression.** Each face is turned and scaled so its five points (eyes, nose tip, mouth corners) sit on reference points, cut to 112 by 112, and given to the expression model of the OpenCV model zoo (MobileFaceNet trained with Progressive Teacher). Its 7 scores become probabilities.
3. **Following.** A face continues the face whose last sighting is nearest, if it moved at most three quarters of its width and changed size at most twice. Readings of one face with the same expression join into a stretch while the gaps are under `gapMs`.

## Licences

The program's own licence is the author's to choose (`loomas plugins check` says there is none yet). What it uses: YuNet is MIT (Shiqi Yu). The expression model is Apache-2.0 (OpenCV model zoo). The data it was trained on is to be confirmed before the plugin is distributed: facial expression collections are often for research only. ONNX Runtime is MIT. The Rust crates are listed in [docs/licences.md](../../docs/licences.md). The models are **not packaged**: they are downloaded.

## Failures

`400` a param is wrong or `media` is not a video Loomas decoded for the plugin. `422` a file cannot be read or a model cannot be loaded. `503` a model could not be downloaded. [plugin.json](plugin.json) lists them with the schemas.
