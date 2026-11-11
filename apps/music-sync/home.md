# Music sync

Finds where a music starts inside a video's sound, and how much faster or slower the video plays it. A Loomas plugin written in Rust, so it is one program with nothing to install.

The use it was made for: a video has its music in low quality, you have the same music in good quality. This plugin says how late the good copy has to start to line up with the video; `media.replaceAudio` (with its `offsetMs`, `speed` and `keep`) then makes a new video with the original pictures and the good music.

## Keeping the video's speed or the music's

When `speed` is not 1 the two cannot both keep their speed. `media.replaceAudio` takes `speed` and `keep`:

- `keep: "video"` (the default): the picture is untouched and the music is stretched to fit it (pitch kept, FFmpeg `atempo`).
- `keep: "audio"`: the music is untouched and the picture is sped up or slowed down to fit it. The picture is made again (H.264, CRF 18), so it takes longer.

## Limits

- One speed for the whole video. A video that cuts the music and joins it again is not handled.
- The speed search is slow: it took 13 s on this 40 s video and grows with the video's length, so a 4-minute video may take a minute or two. It only runs when the plain match is not confirmed.
- The default `minScoreSigma` 30 and the speed search were set from these few synthetic cases, not from a collection of real songs. The first versions of the speed search failed on the degraded copy (steps of 0.003 and the beat of the music did not find it), so a heavily degraded real copy at an odd speed may still be missed: check the result by listening.
- Sounds up to 30 minutes each; it needs memory in proportion (about 1.5 GB at the limit).
- Speech or effects over the music lower the score but rarely move the offset.

## Licences

Rust crates only (`rustfft`, MIT or Apache-2.0, and the shared sound reader), see `docs/licences.md`.
