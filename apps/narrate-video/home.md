# Narrate a video

Narrates a video, or a sound with people talking, in one step: who said what, how loud and in what tone, and how their face looked. A Loomas **plugin made of a flow**: it has no program of its own. Its flow runs five other plugins, and Loomas runs the flow when a step calls the plugin.

```
Person A asked "What is this doing here?"
Person B said in a sad voice: "And I wanted to remind myself to ask you to fix it."
Person C said in an angry voice: "Calm down, both of you."
```

It needs a Loomas core that runs plugins made of flows (loomas-apps/Loomas#107), and these plugins installed, at the version in `requires` of plugin.json or later:

| Plugin | Does |
|---|---|
| speech-to-text | the sentences and when they were said |
| speakers | who speaks when, and the names of known voices |
| voice-tone | how loud each sentence is, and the emotion of the voice |
| face-emotion | the faces in view and their expressions |
| narrator | the sentences of the narration |

## What to expect

- The limits are those of the five plugins: read their pages.
- A face is given to a speaker only when one face is in view for most of the time they talk. In a picture with many people, no face is named.
- **Tested** (2026-11-10) on a synthetic conversation of three Piper voices, as a sound, as a video of a still group picture with that sound, and on a short video with no sound. Not tried on real recordings.
