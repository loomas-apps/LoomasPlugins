# Face expressions

Finds the **faces** in a video, follows each one from picture to picture, and says which **expression** it shows over time: angry, disgust, fear, happy, neutral, sad or surprise. It **never says who a face is**: faces are numbered `face-1`, `face-2`... in the order they show. No language model. A Loomas plugin written in Rust: one program, nothing to install. Loomas decodes two colour pictures a second, 640 pixels wide, and the plugin only looks at them.

## What to expect

- **Tested on one picture only** (2026-11-10): a group selfie (OpenCV's YuNet example) made into a 3-second video. The plugin read the same expressions as a Python reference of the same models (`happy`, 1.0 for the three nearest faces), and found 54 faces where OpenCV found 48 in the picture (the video is compressed, and small faces at the edge of the score come and go). Real videos with people who move, turn away and change expression are not tested yet.
- **Faces that turn away** are lost until they turn back. A face not seen for more than 2 seconds that shows again counts as a new face.
- **Small faces.** Under 24 pixels wide (in a 640-wide picture), there are too few pixels to read an expression, so the face is followed but gets none.
- **Speed.** 3 seconds of video took under 3 s, with the start of the plugin, on a graphics card. The detector looks at every picture, and the expression model at every face of at least `minFaceWidth`.
- **The first call downloads the models** (230 kB and 4.8 MB) and keeps them in `~/.loomas/models/face-emotion` (or in the folder `LOOMAS_MODELS` names), after checking their size and sha256.

## Privacy

Faces are personal data. The plugin keeps nothing: no face picture, no face print. It gives only times, boxes and expression names, and it cannot tell who a face is or match a face across videos. An expression is what a face shows, not what a person feels.

## Licences

The program's own licence is the author's to choose (`loomas plugins check` says there is none yet). What it uses: YuNet is MIT (Shiqi Yu). The expression model is Apache-2.0 (OpenCV model zoo). The data it was trained on is to be confirmed before the plugin is distributed: facial expression collections are often for research only. ONNX Runtime is MIT. The Rust crates are listed in docs/licences.md. The models are **not packaged**: they are downloaded.
