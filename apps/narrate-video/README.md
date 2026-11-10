# narrate-video

Narrates a video, or a sound with people talking, in one step: who said what, how loud and in what tone, and how their face looked. A Loomas **plugin made of a flow**: it has no program of its own. Its [flow](flow.loom) runs five other plugins, and Loomas runs the flow when a step calls the plugin.

```
Person A asked "What is this doing here?"
Person B said in a sad voice: "And I wanted to remind myself to ask you to fix it."
Person C said in an angry voice: "Calm down, both of you."
```

It needs a Loomas core that runs plugins made of flows (loomas-apps/Loomas#107), and these plugins installed, at the version in `requires` of [plugin.json](plugin.json) or later:

| Plugin | Does |
|---|---|
| [speech-to-text](../speech-to-text) | the sentences and when they were said |
| [speakers](../speakers) | who speaks when, and the names of known voices |
| [voice-tone](../voice-tone) | how loud each sentence is, and the emotion of the voice |
| [face-emotion](../face-emotion) | the faces in view and their expressions |
| [narrator](../narrator) | the sentences of the narration |

## `narrate`

| Input | Read |
|---|---|
| `media` | the file to narrate: a video, or a sound (required) |
| `model` | the speech-to-text model (default `base.en`) |
| `language` | the language people speak, as a code (default `en`) |
| `voices` | voice folders or patterns of known speakers, so they are named (see [speakers](../speakers)); default none |

It gives what [narrator](../narrator) gives: `lines` (timed text, one for each sentence), `text` (the lines one after another) and `people` (each speaker, what the narration calls them, and their face).

```json
{ "Narrate": { "type": "narrate-video.narrate", "inputs": { "media": "@triggerBody().path" } } }
```

## How it works

The flow opens the file. When it has sound, it runs speech-to-text, speakers and voice-tone. When it has a picture, it runs face-emotion. Then narrator joins what they found. A file with no sound gives no lines, because the narrator writes a line for each thing said. To change what it does, edit the flow, or copy it, and make your own plugin with `loomas plugins assemble <flow> --name <name> --version <version>`.

## What to expect

- The limits are those of the five plugins: read their pages.
- A face is given to a speaker only when one face is in view for most of the time they talk. In a picture with many people, no face is named.
- **Tested** (2026-11-10) on a synthetic conversation of three Piper voices, as a sound, as a video of a still group picture with that sound, and on a short video with no sound. Not tried on real recordings.

## Failures

`400` an input is wrong. Any failure of a step inside fails the step that called the plugin, with the name of the inner step (`Words`, `Who`, `Tone`, `Faces`, `Tell`) and its error.
