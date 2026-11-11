# Sound volume

How loud a sound is, over time, and in which frequencies. A Loomas plugin written in Rust, so it is one program with nothing to install: it reads mp3, wav, flac, ogg, aac and alac (and the sound of an mp4) by itself.

## Notes

- The windows follow one another with none missing or overlapping. A last window shorter than a quarter of the others is left out.
- Resampling is a simple average when going down and a straight line when going up: good enough to measure a sound, not for music production.
- The whole sound is read into memory (a 10 minute stereo mp3 is about 100 MB of samples).
