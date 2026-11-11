# Read the text on the screen

Reads the **text a video shows**: titles, subtitles burnt into the picture, signs, captions. It says what is written, when it is on the screen and where. A Loomas plugin written in Rust: one program, nothing to install. It uses the PaddleOCR networks, which run on the computer, so the video goes nowhere. The networks are downloaded once, the first time.

It is the other half of "media to text": speech-to-text writes down what is said, and this writes down what is shown.

## Scripts

One script is read at a time, because each recognizer knows the characters of one script. A video with two scripts (an English title and Korean subtitles) needs two calls.

| `language` | Reads | Download |
|---|---|---|
| `latin` | Latin letters: Portuguese, Spanish, French, German, Italian, Dutch, Polish, Turkish and about forty more | 7.5 MB |
| `english` | English only (a little more exact on English than the Latin model) | 7.5 MB |
| `chinese` | Chinese, Japanese and English | 80.6 MB |
| `korean` | Korean and English | 12.8 MB |
| `arabic` | Arabic, Persian and Urdu | 8.6 MB |
| `eslav` | Cyrillic: Russian, Ukrainian, Belarusian, Bulgarian and others | 7.5 MB |
| `greek` | Greek | 7.4 MB |
| `hindi` | Devanagari: Hindi, Marathi, Nepali | 8.6 MB |
| `thai` | Thai | 7.5 MB |
| `tamil` | Tamil | 8.6 MB |
| `telugu` | Telugu | 8.6 MB |

The Latin dictionary spells some letters as two characters (the UTF-8 of `í` read as Latin-1), and the network sometimes writes a letter that way. The plugin puts such pairs back as the letter.

## Be honest about what it cannot do

- **One picture a second.** A text on the screen for less than a second can be missed.
- **Printed text in straight lines.** Handwriting, text on a curve and text turned on its side are read badly. The boxes of the lines are upright rectangles.
- **Small, blurred or low-contrast text** is missed or misread. The `accurate` detector finds more of it.
- **A mix of scripts in one line** is read with the characters of the one script you chose, so the other script comes out as wrong characters.
- **Chinese lines can be split** where there is a wide gap (after a comma), into two entries with the same times.
- **Tested on made-up pictures only:** lines of Portuguese, English, Chinese and Korean text on a plain background, which it read exactly. It was not tested on real videos, so how often it is wrong on a real subtitle track, a news ticker or a street sign is not known.
