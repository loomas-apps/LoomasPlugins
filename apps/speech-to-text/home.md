# Speech to text

The sentences said in a video or a sound, with the time each one starts and ends. A Loomas plugin written in **Python**, because that is where the speech recognition is: it uses [faster-whisper](https://github.com/SYSTRAN/faster-whisper) (the Whisper models on CTranslate2), which runs on an ordinary processor and reads the sound of most video and audio files by itself.

## Notes

- Only one call is served at a time (`concurrency` 1): a model uses all the processor cores already.
- The speech is recognised on the processor with 8-bit numbers (`int8`). A graphics card would be faster; it is not used yet.
- To try it on a real recording, set `LOOMASPLUGINS_SPEECH_SAMPLE` to a video or sound with English speech and run `smith test -p speech-to-text`: one test then runs a real model (it is skipped otherwise).
- Names, numbers and punctuation are what the model writes; check anything that matters.
