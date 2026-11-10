# narrator

Writes what other plugins found in a video as a **timeline of plain sentences**: who said what, how they said it (loud, quiet, the emotion of the voice), how their face looked, what moved, and what was in view. **No model of any kind**: the narrator only joins the findings by time and fills templates, so the same findings always give the same text. A Loomas plugin written in Rust: one program, nothing to install, nothing to download.

```
Person A asked in a sad voice: "What is this doing here?"
Person B said "I left it there yesterday because it ripped."
Person A shouted in an angry voice, looking angry: "I already told you not to leave this mess in the house!" At the same time, a person moves an arm fast. On screen: a person holding trousers.
A person falls.
```

## `narrate`

Every list is optional. Give each one as its plugin gives it.

| Input | From | Read |
|---|---|---|
| `sentences` | [speech-to-text](../speech-to-text) `sentences` | `startMs`, `endMs`, `text` |
| `turns` | [speakers](../speakers) `turns` | `startMs`, `endMs`, `label` |
| `tones` | [voice-tone](../voice-tone) `segments` | `startMs`, `endMs`, `loudness`, `emotion`, `emotionConfidence` |
| `expressions` | [face-emotion](../face-emotion) `expressions` | `startMs`, `endMs`, `face`, `label`, `confidence` |
| `movements` | [movement](../movement) `movements` | `startMs`, `endMs`, `label` |
| `scenes` | [visual-phrases](../visual-phrases) or [coco-objects](../coco-objects) `appearances` | `startMs`, `endMs`, `phrase` or `class` |
| `minConfidence` | | an emotion or expression less sure than this is not written, 0 to 1 (default 0.5) |
| `answerWithinMs` | | how soon after another person's question a line counts as its answer (default 3000) |

It gives `{ lines, text, people }`:

- `lines` are `{ startMs, endMs, text, speaker }`, so they show as text on the timeline. There is one line for each sentence, or for each turn when there are no sentences. There is also one for each movement or thing in view while nobody speaks (`speaker` is null for those).
- `text` is the lines, one after another.
- `people` gives, for each speaker, what the narration calls them and which face was taken for theirs: `{ "speaker": "voice-1", "name": "Person A", "face": "face-1" }`.

```json
{ "Tell": { "type": "narrator.narrate",
            "runAfter": { "Who": ["Succeeded"], "Tone": ["Succeeded"], "Faces": ["Succeeded"] },
            "inputs": { "sentences": "@body('Words').sentences", "turns": "@body('Who').turns",
                        "tones": "@body('Tone').segments", "expressions": "@body('Faces').expressions" } } }
```

## How it writes

- **Who.** The speaker of a sentence is the turn that shares most of its time. A name given in `speakers` stays. The ids `voice-1`, `voice-2`... become "Person A", "Person B"... in the order they first speak. A sentence with no turn is said by "Someone".
- **The verb.** A sentence that ends in `?` is *asked*. One right after another person's question is *answered*. Anything else is *said*. A `loud` voice makes *said* into *shouted* and the others *asked loudly* or *answered loudly*. A `low` voice gives *whispered*, or *asked quietly* and *answered quietly*.
- **The voice.** The emotion of the tone that shares most time with the sentence: "in an angry voice", "in a sad voice"... Neutral and calm are not written, and neither is anything under `minConfidence`.
- **The face.** A speaker's face is the face of `face-emotion` in view for at least half the time they talk. When two faces are in view about as long, there is no face, because the narrator does not guess. Then the expression of that face during the sentence: "looking angry"... Neutral is not written.
- **Movements.** The `movement` plugin does not know whose body moved belongs to which voice, so a movement is always "a person": "At the same time, a person moves an arm fast."
- **In view.** Up to three phrases or classes that share time with the sentence: "On screen: a person holding trousers." The words are the plugin's own: write the phrases of `visual-phrases` the way you want them read.

## What to expect

- **Timing decides everything.** A person who talks off screen while another face is in view can be given that face. A sentence cut across two speakers goes to the one who said most of it.
- **English only.** The templates are English, whatever the language of the sentences.
- **Tested** (2026-11-10) on unit examples and on a whole chain over a synthetic conversation of three Piper voices: `speech-to-text`, then `speakers`, then `voice-tone` and the narrator. Every line went to the right person, and the line made 10 dB louder came out *shouted*. The emotions of the synthetic voices are not reliable, so the voice phrases of that test say little.

## Failures

`400` a param is wrong, or an item of a list lacks `startMs`, `endMs` or the field the narrator reads. [plugin.json](plugin.json) lists the schemas.
