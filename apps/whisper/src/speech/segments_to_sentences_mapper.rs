use serde_json::{json, Value};

/// One token of a segment as whisper.cpp gives it: its text, how sure the model was, and its
/// times in centiseconds from the start of the sound it heard.
#[derive(Debug, Clone, PartialEq)]
pub struct HeardToken {
    pub text: String,
    pub probability: f32,
    pub start_cs: i64,
    pub end_cs: i64,
}

/// One segment as whisper.cpp gives it: its times in centiseconds, its text, how likely it is that
/// nobody spoke, and its tokens.
#[derive(Debug, Clone, PartialEq)]
pub struct HeardSegment {
    pub start_cs: i64,
    pub end_cs: i64,
    pub text: String,
    pub no_speech: f32,
    pub tokens: Vec<HeardToken>,
}

/// What `sentences_of` keeps and how it places the times.
#[derive(Debug, Clone, Copy)]
pub struct Placing {
    /// Where the sound heard starts in the source, in milliseconds.
    pub offset_ms: u64,
    /// How long the sound heard lasts, in milliseconds; no time goes past its end.
    pub duration_ms: u64,
    /// Give each sentence its words.
    pub words: bool,
    /// Leave out a segment the model thinks is most likely silence.
    pub skip_silence: bool,
}

/// A segment the model thinks is silence more often than this is left out with `skip_silence`.
const SILENCE: f32 = 0.6;
/// The shortest a sentence or a word lasts, so its end is always after its start.
const SHORTEST_MS: u64 = 10;

fn is_text(token: &HeardToken) -> bool {
    !token.text.is_empty() && !token.text.starts_with("[_") && !token.text.starts_with("<|")
}

fn mean_probability<'a>(tokens: impl Iterator<Item = &'a HeardToken>) -> f64 {
    let (sum, count) = tokens.fold((0.0, 0), |(sum, count), token| {
        (sum + f64::from(token.probability), count + 1)
    });
    if count == 0 {
        0.0
    } else {
        (sum / f64::from(count) * 1000.0).round() / 1000.0
    }
}

/// `start..end` in centiseconds as milliseconds of the source: moved by the offset, kept inside the
/// sound, and at least `SHORTEST_MS` long.
fn span(start_cs: i64, end_cs: i64, placing: &Placing) -> (u64, u64) {
    let last = placing.offset_ms + placing.duration_ms;
    let at = |cs: i64| (placing.offset_ms + u64::try_from(cs.max(0)).unwrap_or(0) * 10).min(last);
    let start = at(start_cs)
        .min(last.saturating_sub(SHORTEST_MS))
        .max(placing.offset_ms);
    (start, at(end_cs).max(start + SHORTEST_MS))
}

/// The words of a segment: a token that starts with a space starts a new word.
fn words_of(segment: &HeardSegment, placing: &Placing) -> Vec<Value> {
    let mut groups: Vec<Vec<&HeardToken>> = Vec::new();
    for token in segment.tokens.iter().filter(|token| is_text(token)) {
        match groups.last_mut() {
            Some(group) if !token.text.starts_with(' ') => group.push(token),
            _ => groups.push(vec![token]),
        }
    }
    groups
        .into_iter()
        .filter_map(|group| {
            let text: String = group.iter().map(|token| token.text.as_str()).collect::<String>().trim().to_string();
            if text.is_empty() {
                return None;
            }
            let (start, end) = span(group[0].start_cs, group[group.len() - 1].end_cs, placing);
            Some(json!({ "startMs": start, "endMs": end, "text": text, "confidence": mean_probability(group.into_iter()) }))
        })
        .collect()
}

/// The sentences of the segments, in the shape of the speech-to-text plugin: `{ startMs, endMs,
/// text, confidence }`, with `words` when asked. Empty segments are left out, and so is silence
/// when `skip_silence` asks for it. Times are milliseconds of the source.
pub fn sentences_of(segments: &[HeardSegment], placing: &Placing) -> Vec<Value> {
    segments
        .iter()
        .filter(|segment| !(placing.skip_silence && segment.no_speech > SILENCE))
        .filter_map(|segment| {
            let text = segment.text.trim();
            if text.is_empty() {
                return None;
            }
            let (start, end) = span(segment.start_cs, segment.end_cs, placing);
            let mut sentence = json!({
                "startMs": start,
                "endMs": end,
                "text": text,
                "confidence": mean_probability(segment.tokens.iter().filter(|token| is_text(token))),
            });
            if placing.words {
                sentence["words"] = Value::Array(words_of(segment, placing));
            }
            Some(sentence)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(text: &str, probability: f32, start_cs: i64, end_cs: i64) -> HeardToken {
        HeardToken {
            text: text.to_string(),
            probability,
            start_cs,
            end_cs,
        }
    }

    fn placing(words: bool) -> Placing {
        Placing {
            offset_ms: 60_000,
            duration_ms: 30_000,
            words,
            skip_silence: true,
        }
    }

    #[test]
    fn segments_become_sentences_in_the_time_of_the_source_with_their_words() {
        let segments = [HeardSegment {
            start_cs: 100,
            end_cs: 250,
            text: " Hello there.".to_string(),
            no_speech: 0.1,
            tokens: vec![
                token("[_BEG_]", 1.0, 100, 100),
                token(" Hello", 0.9, 100, 150),
                token(" there", 0.7, 150, 220),
                token(".", 0.8, 220, 250),
            ],
        }];
        let sentences = sentences_of(&segments, &placing(true));
        assert_eq!(sentences[0]["startMs"], 61_000);
        assert_eq!(sentences[0]["endMs"], 62_500);
        assert_eq!(sentences[0]["text"], "Hello there.");
        assert_eq!(sentences[0]["confidence"], 0.8);
        let words: Vec<&str> = sentences[0]["words"]
            .as_array()
            .unwrap()
            .iter()
            .map(|w| w["text"].as_str().unwrap())
            .collect();
        assert_eq!(words, ["Hello", "there."]);
        assert!(sentences_of(&segments, &placing(false))[0]
            .get("words")
            .is_none());
    }

    #[test]
    fn a_sentence_always_ends_after_it_starts_and_empty_or_silent_ones_are_left_out() {
        let segment = |start_cs, end_cs, text: &str, no_speech| HeardSegment {
            start_cs,
            end_cs,
            text: text.to_string(),
            no_speech,
            tokens: vec![],
        };
        let sentences = sentences_of(
            &[
                segment(500, 500, "Yes", 0.0),
                segment(600, 700, "  ", 0.0),
                segment(800, 900, "hmm", 0.9),
                segment(3000, 3200, "End", 0.0),
            ],
            &placing(false),
        );
        assert_eq!(sentences.len(), 2);
        assert!(sentences[0]["endMs"].as_u64() > sentences[0]["startMs"].as_u64());
        // Past the end of the sound heard: kept inside it, still ending after it starts.
        assert_eq!(sentences[1]["endMs"], 90_000);
        assert!(sentences[1]["startMs"].as_u64().unwrap() < 90_000);
    }
}
