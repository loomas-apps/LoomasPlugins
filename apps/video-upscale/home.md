# Upscale video

Makes a video larger, two or four times on each side, with a [Real-ESRGAN](https://github.com/xinntao/Real-ESRGAN) network that adds detail, or with plain Lanczos scaling. A Loomas plugin written in Rust. It uses **no language model**. It runs on the graphics card when the host grants one.

## The models

| Model | Size | For | Piece size |
|---|---|---|---|
| `general-x4v3` | 4.9 MB | most video; small and fast | any (`tile`) |
| `animevideo-v3` | 2.5 MB | cartoons and anime | 256, fixed |
| `x4plus` | 67 MB | photographs; the most detail and the slowest | 128, fixed |
| `lanczos` | none | a baseline that adds no detail, and needs no download | |

Real-CUGAN, another network for anime, is not here: no ONNX file of it could be verified.

## What to know

- **No sound.** The plugin gets pictures, not the source file, so the new video is silent. Add the sound of the source in a later step of the flow.
- **Every picture is decoded.** A 1080p picture is 6 MB, so a minute of video is about 11 GB for the host to decode, and Loomas refuses a decode that is too large. Use a short clip, or a stretch of a video.
- **Slow.** A network makes four pixels for each pixel of the source: a 1080p picture becomes 4K. Use the graphics card (`device`), and a larger `tile` for `general-x4v3` if it has the memory.
- **Pieces.** A picture is cut into overlapping square pieces; each answer loses 16 pixels of edge, so no seam shows. Fixed-size networks (`animevideo-v3`, `x4plus`) have the size built in.
- **FFmpeg.** The plugin starts FFmpeg to write the file: it uses `LOOMAS_FFMPEG` (a folder with `ffmpeg` in it, or the program) or the system path, and needs the `libx264` encoder. The frame rate is the average of the video's.
- **The first call downloads the model** from Hugging Face, checks its sum, and keeps it in `~/.loomas/models/video-upscale` (or in the folder `LOOMAS_MODELS` names).

## Licences

The program is MIT. Real-ESRGAN code and weights are BSD-3-Clause; ONNX Runtime is MIT; FFmpeg is the user's own. The weights are **not packaged**: they are downloaded.
