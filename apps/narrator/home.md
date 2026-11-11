# Narrator

Writes what other plugins found in a video as a **timeline of plain sentences**: who said what, how they said it (loud, quiet, the emotion of the voice), how their face looked, what moved, and what was in view. **No model of any kind**: the narrator only joins the findings by time and fills templates, so the same findings always give the same text. A Loomas plugin written in Rust: one program, nothing to install, nothing to download.

```
Person A asked in a sad voice: "What is this doing here?"
Person B said "I left it there yesterday because it ripped."
Person A shouted in an angry voice, looking angry: "I already told you not to leave this mess in the house!" At the same time, a person moves an arm fast. On screen: a person holding trousers.
A person falls.
```

## What to expect

- **Timing decides everything.** A person who talks off screen while another face is in view can be given that face. A sentence cut across two speakers goes to the one who said most of it.
- **English only.** The templates are English, whatever the language of the sentences.
- **Tested** (2026-11-10) on unit examples and on a whole chain over a synthetic conversation of three Piper voices: `speech-to-text`, then `speakers`, then `voice-tone` and the narrator. Every line went to the right person, and the line made 10 dB louder came out *shouted*. The emotions of the synthetic voices are not reliable, so the voice phrases of that test say little.
