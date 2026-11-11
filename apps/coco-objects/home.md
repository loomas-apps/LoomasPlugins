# Find objects

Finds the things a video shows, from the 80 classes of the [COCO](https://cocodataset.org) set (person, cat, dog, car, bottle, chair...), with a detector and **no language model**. A Loomas plugin written in Rust: one program, nothing to install. Loomas decodes the pictures (colour, 640 pixels wide, one every half second) and the plugin only looks at them.

It is the fast, narrow kind of vision plugin: cheap on a CPU, precise on what it knows, and it knows only the 80 classes. A broader one (any phrase) or one that describes a picture would be another plugin.

## What to know

- **Half a second apart.** The plugin looks at two pictures a second. Something on screen for less than that can be missed, and `minSamples` (2 by default) asks for two pictures, so set it to 1 to report a thing seen once.
- **COCO is 80 classes** and no more: it knows "cat" and "dog" but not which breed, "person" but not who, and nothing outside the list.
- **Which model.** `nano` takes a few milliseconds a picture on a laptop CPU and misses small or partly hidden things; `s` is about eight times slower and finds more. Measured with ONNX Runtime on 16 cores: nano about 6 ms, s about 40 ms for one picture.
- **The first call downloads the model** (nano 3.6 MB, tiny 20 MB, s 36 MB) from the release of the detector's authors, checks its sum, and keeps it in `~/.loomas/models/coco-objects` (or in the folder `LOOMAS_MODELS` names). Without a network and without the file, the call fails with a 503 and can be tried again. The plugin declares the network hosts it uses in `permissions`.
- Whole videos only: it does not declare windows, because a thing that stays across two windows would be reported twice.

## Licences

The program's own licence is the author's to choose (`loomas plugins check` says there is none yet). What it uses: the YOLOX code and weights are Apache-2.0; ONNX Runtime is MIT; the Rust crates are listed in docs/licences.md. The weights are **not packaged**: they are downloaded from their authors.
