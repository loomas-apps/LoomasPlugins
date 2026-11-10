use super::voice_pattern_contract::VoicePattern;

/// The cosine similarity of two prints of length 1.
pub fn similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Which pattern each voice heard is: the pair with the highest similarity is matched first, then
/// the next among those left, so a pattern names at most one voice and a voice gets at most one
/// pattern. A pair below `min_similarity` is no match. Gives, for each voice, the index of its
/// pattern and the similarity.
pub fn match_voices(
    voices: &[Vec<f32>],
    patterns: &[VoicePattern],
    min_similarity: f32,
) -> Vec<Option<(usize, f32)>> {
    let mut pairs: Vec<(f32, usize, usize)> = voices
        .iter()
        .enumerate()
        .flat_map(|(voice, print)| {
            patterns
                .iter()
                .enumerate()
                .map(move |(pattern, known)| (similarity(print, &known.print), voice, pattern))
        })
        .filter(|(score, _, _)| *score >= min_similarity)
        .collect();
    pairs.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut matched = vec![None; voices.len()];
    let mut taken = vec![false; patterns.len()];
    for (score, voice, pattern) in pairs {
        if matched[voice].is_none() && !taken[pattern] {
            matched[voice] = Some((pattern, score));
            taken[pattern] = true;
        }
    }
    matched
}

#[cfg(test)]
mod tests {
    use super::*;

    fn known(angle: f32) -> VoicePattern {
        VoicePattern { id: format!("v{angle}"), name: String::new(), seconds: 1.0, print: vec![angle.cos(), angle.sin()] }
    }

    #[test]
    fn the_best_pairs_match_first_and_each_pattern_once() {
        let voices = vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![0.995, 0.0998]];
        let patterns = [known(0.05), known(1.6)];
        let matched = match_voices(&voices, &patterns, 0.5);
        assert_eq!(matched[0].map(|m| m.0), Some(0));
        assert_eq!(matched[1].map(|m| m.0), Some(1));
        assert_eq!(matched[2], None, "the pattern was taken by a closer voice");
    }

    #[test]
    fn a_voice_below_the_least_similarity_stays_unknown() {
        let matched = match_voices(&[vec![1.0, 0.0]], &[known(1.2)], 0.5);
        assert_eq!(matched, vec![None]);
    }
}
