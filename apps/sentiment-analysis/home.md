# Rate how positive or negative the words are

Says how **positive, neutral or negative** each sentence of a transcript is, with a model that runs on the computer and **no language model**. A Loomas plugin written in Rust: one program, nothing to install. Loomas loads the plugin when a flow calls it; the model is downloaded once, the first time.

It rates the **words**. It does not hear the voice, and it does not know what the words mean to the person who said them: read "Be honest about what it cannot do" below before you build on it.

## Be honest about what it cannot do

- **It rates wording, not distress.** On the test below "Me ajuda, por favor, estou com muito medo!" ("Help me, please, I am very afraid!") is rated **positive** (0.90): the polite words weigh more than the fear. A sentiment model is not a way to find a person in danger. The `crisis-moments` plugin does not use it for that.
- **Sarcasm and irony are not understood.** "Great, another Monday" reads as positive.
- **It was trained on short texts** (reviews and posts). Spoken language, with its pauses and its mistakes in the transcript, is harder, and the transcript's own mistakes pass straight into the rating.
- **Neutral is weak.** A plain statement ("The table is in the room") gets a low `confidence` (0.44) and may come out positive. Treat a low `confidence` as "no opinion".
- **The model rates one sentence at a time.** It does not know what was said before. A text of more than 512 pieces is cut.

Tested on seven sentences (English and Portuguese) on a Windows computer: the clear ones were right (positive 0.91 and 0.97, negative 0.99), the fear sentence and the plain statement were not, as said above. The seven sentences took 7.5 s the first time on the processor, including loading the model. This is a small test, not a measurement of accuracy. It was not run on a graphics card.
