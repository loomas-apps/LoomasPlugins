# Speech to text (whisper.cpp)

Writes down **what is said** in a video or a sound, with the time each sentence starts and ends, using [whisper.cpp](https://github.com/ggml-org/whisper.cpp). It does the same job as speech-to-text and takes and gives the same things, so a flow can use either. The difference is where it runs: this plugin uses a **graphics card through Vulkan**, from AMD, NVIDIA or Intel, and the processor when there is no card. speech-to-text uses an NVIDIA card or the processor only. A Loomas plugin written in Rust: one program, no Python.

## Which model

Measured on 2026-11-11, on a video of 3 min 44 s in Portuguese, with people talking over music, on an AMD Radeon RX 7900 XT and a 16-thread processor. The times include loading the model:

| Model | Where | Time | What it heard |
|---|---|---|---|
| `base` | the card | 4.4 s | music only (`[Música]`) |
| `large-v3-turbo` | the card | 9.8 s | the speech, correctly, with a few repeated shouts |
| `base` | the processor | 14.9 s | music only |
| `base.en` in speech-to-text | the processor | 22 s | English words that were not said |

On 36 s of clear English speech with `base.en` on the processor, it takes 3.5 s, and speech-to-text 7.4 s.

So, for a language other than English or for speech over noise, use `large-v3-turbo` on a card (1.6 GB, downloaded once). The small models are fast but miss speech under music. Always give `language` when you know it.

## What to expect

- **Repetition.** Whisper can repeat a word or a shout many times in a long, noisy stretch. Read the confidence of each sentence.
- **Threads.** On the processor it uses one thread for each physical core (half the logical processors, at most 8). With every logical processor it was four times slower.
- **The card's name** is not given yet (`device.name` is null). whisper.cpp writes it in its log.
- **Tested** only on Windows, with an AMD card. Not tried on Linux, on NVIDIA or Intel cards, or on macOS.
