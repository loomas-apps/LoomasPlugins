# Find what a phrase describes

Finds when a phrase describes what a video shows ("two cats sleeping on a couch", "a red car", "a person skiing"), with an image-text model and **no language model**. A Loomas plugin written in Rust: one program, nothing to install. Loomas decodes the pictures (colour, 640 pixels wide, one a second) and the plugin only looks at them.

It is the broader kind of vision plugin: it has no fixed list of classes, so anything you can say in words can be asked for, at the price of being slower and less sure than coco-objects, which knows 80 classes and draws boxes. A flow can try the cheap detector first and use this one on what it left over. Measured with ONNX Runtime on a laptop CPU, a picture takes about 30 ms, so an hour of video (one picture a second) is a couple of minutes.

## What to know

- **The scores are low.** This kind of model scores each phrase on its own, between 0 and 1, and is stingy: on the photo of two cats from the model's own card, "a photo of 2 cats" scored 0.20 here and a plain "a cat" 0.002. That is why `minConfidence` is 0.1 and not 0.5. A phrase that describes the whole picture ("two cats sleeping on a couch", "a photo of two cats") scores far better than a single word. Try a few wordings.
- **What it is good at:** scenes, things, colours and kinds of place. **What it is not:** counts ("exactly three people"), negations ("no people"), small things in a big picture, text in the picture, and who someone is. The picture is squeezed to 224 x 224 for the model, so detail is lost.
- **A second apart.** The plugin looks at one picture a second. Something on screen for less than that can be missed; `minSamples` (1 by default) asks for more pictures in a row when a single one is too much.
- **Phrases compete with nothing.** Each phrase is scored on its own, so 16 phrases cost one more reading of the phrase each and no more looking.
- **The first call downloads the model** (about 205 MB: the picture model, the phrase model and the tokenizer) and keeps it in `~/.loomas/models/visual-phrases` (or in the folder `LOOMAS_MODELS` names), after checking each file's size and sha256. Without a network and without the files, the call fails with a 503 and can be tried again. The plugin declares the network hosts it uses in `permissions`. The 8-bit version is used, which is smaller and faster than the full one; how much score it loses against the full one has not been measured here (issue to check).
- Whole videos only: it does not declare windows, because a phrase that stays true across two windows would be reported twice.

## Licences

The program's own licence is the author's to choose (`loomas plugins check` says there is none yet). What it uses: SigLIP is Apache-2.0 (Google; the conversion says the same); ONNX Runtime is MIT; the Hugging Face `tokenizers` crate is Apache-2.0; the Rust crates are listed in docs/licences.md. The model is **not packaged**: it is downloaded.
