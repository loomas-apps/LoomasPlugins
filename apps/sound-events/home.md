# Find sounds

Finds the sounds a video has, from the 521 classes of [AudioSet](https://research.google.com/audioset/) (speech, music, cat, meow, dog, bark, siren, door...), with a small neural tagger and **no language model**. A Loomas plugin written in Rust: one program, nothing to install. Loomas decodes the sound (16 kHz, one channel) and the plugin only listens to it.

It is the fast, narrow kind of sound plugin, the counterpart of coco-objects for pictures: cheap on a CPU, good on what it knows, and it knows only the 521 classes. Measured with ONNX Runtime on a laptop CPU, 8 seconds of sound take 11 ms and 70 seconds 65 ms, so an hour of video is a few seconds of listening.

## What to know

- **About half a second.** The tagger scores a frame of about a second every half second, and a sound is heard in every frame it touches. Each frame is taken as the half second round its middle, so an event starts and ends within about half a second of the sound. On a meow that really lasts from 5.0 to 6.15 s it gave 4.6 to 6.0 s.
- **521 classes** and no more. It tells a meow from a bark, not one cat from another, nor what is said (that is a speech plugin).
- **A pure tone is not silence to it**: a steady 440 Hz sine was tagged *Alarm*, *Telephone*, *Dial tone* and *Busy signal* for its four seconds. A tagger gives its best guess among the classes it has.
- **The first call downloads the model** (16 MB) and keeps it in `~/.loomas/models/sound-events` (or in the folder `LOOMAS_MODELS` names), after checking its size and sha256. Without a network and without the file, the call fails with a 503 and can be tried again. The plugin declares the network hosts it uses in `permissions`.
- Whole videos only: it does not declare windows, because a sound that goes on across two windows would be reported twice. A long sound is scored five minutes at a time, so its memory does not grow with the video.

## Licences

The program's own licence is the author's to choose (`loomas plugins check` says there is none yet). What it uses: YAMNet is Apache-2.0 (the ONNX conversion says the same); ONNX Runtime is MIT; the Rust crates are listed in docs/licences.md. The model is **not packaged**: it is downloaded.
