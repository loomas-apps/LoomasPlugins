/// The voice print of a run of one local speaker of one window.
pub struct Piece {
    /// The window the run is in.
    pub window: usize,
    /// The local speaker of the window. Pieces of two local speakers of the same window are two
    /// different voices, so they never join.
    pub local: usize,
    /// The print, of length 1.
    pub print: Vec<f32>,
}

/// Two groups of prints join while the cosine similarity of their mean prints is at least this.
/// Prints of one voice measured 0.56 to 0.87 and of two voices at most 0.2 (synthetic voices,
/// 2026-11-10). Pyannote's Euclidean threshold (0.7045 on unnormalised means) kept one voice in
/// three groups when a voice has only a few pieces, as it has with windows 5 seconds apart.
pub const JOIN_SIMILARITY: f32 = 0.6;
const JOIN_DISTANCE: f32 = 1.0 - JOIN_SIMILARITY;

struct Group {
    sum: Vec<f32>,
    /// The window and local speaker of its pieces, sorted.
    slots: Vec<(usize, usize)>,
}

impl Group {
    /// One less the cosine similarity of the two mean prints; infinite when they share a window.
    fn distance(&self, other: &Group) -> f32 {
        if speaks_apart(&self.slots, &other.slots) {
            return f32::INFINITY;
        }
        let length = |sum: &[f32]| sum.iter().map(|value| value * value).sum::<f32>().sqrt();
        let lengths = length(&self.sum) * length(&other.sum);
        if lengths == 0.0 {
            return 1.0;
        }
        1.0 - self.sum.iter().zip(&other.sum).map(|(a, b)| a * b).sum::<f32>() / lengths
    }
}

/// Whether the two groups hold two different local speakers of one window: the speech model said
/// those are two people. Both lists are sorted.
fn speaks_apart(a: &[(usize, usize)], b: &[(usize, usize)]) -> bool {
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].0.cmp(&b[j].0) {
            std::cmp::Ordering::Equal => {
                let window = a[i].0;
                let locals = |list: &[(usize, usize)], from: usize| {
                    list[from..].iter().take_while(|slot| slot.0 == window).map(|slot| slot.1).collect::<Vec<_>>()
                };
                let (mine, theirs) = (locals(a, i), locals(b, j));
                if mine.iter().any(|local| theirs.iter().any(|other| other != local)) {
                    return true;
                }
                i += mine.len();
                j += theirs.len();
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
    false
}

/// Puts the pieces into voices by agglomerative clustering with centroid linkage: the two closest
/// groups join while their mean prints are at least `JOIN_SIMILARITY` alike, and, when
/// `max_voices` is given, until
/// there are no more voices than that. Pieces of two local speakers of one window never join.
/// Gives the voice
/// of each piece; voices are numbered from 0 in the order their first piece comes.
pub fn cluster_voices(pieces: &[Piece], max_voices: Option<usize>) -> Vec<usize> {
    let mut groups: Vec<Option<Group>> = pieces
        .iter()
        .map(|piece| {
            Some(Group {
                sum: piece.print.clone(),
                slots: vec![(piece.window, piece.local)],
            })
        })
        .collect();
    let mut owner: Vec<usize> = (0..pieces.len()).collect();
    let mut nearest: Vec<(f32, usize)> = (0..groups.len())
        .map(|i| closest_to(&groups, i))
        .collect();
    let mut alive = groups.len();
    loop {
        let best = (0..groups.len())
            .filter(|&i| groups[i].is_some())
            .min_by(|&a, &b| nearest[a].0.total_cmp(&nearest[b].0));
        let Some(i) = best else { break };
        let (distance, j) = nearest[i];
        let wanted = distance < JOIN_DISTANCE || max_voices.is_some_and(|max| alive > max);
        if !distance.is_finite() || !wanted {
            break;
        }
        let absorbed = groups[j].take().expect("a nearest group is alive");
        let kept = groups[i].as_mut().expect("the best group is alive");
        for (sum, value) in kept.sum.iter_mut().zip(&absorbed.sum) {
            *sum += value;
        }
        kept.slots.extend(absorbed.slots);
        kept.slots.sort_unstable();
        for piece_owner in &mut owner {
            if *piece_owner == j {
                *piece_owner = i;
            }
        }
        alive -= 1;
        nearest[i] = closest_to(&groups, i);
        for k in 0..groups.len() {
            if k == i || groups[k].is_none() {
                continue;
            }
            if nearest[k].1 == i || nearest[k].1 == j {
                nearest[k] = closest_to(&groups, k);
            } else {
                let to_kept = groups[k]
                    .as_ref()
                    .expect("checked above")
                    .distance(groups[i].as_ref().expect("kept is alive"));
                if to_kept < nearest[k].0 {
                    nearest[k] = (to_kept, i);
                }
            }
        }
    }
    let mut numbers = std::collections::HashMap::new();
    owner
        .iter()
        .map(|group| {
            let next = numbers.len();
            *numbers.entry(*group).or_insert(next)
        })
        .collect()
}

/// The distance from group `i` to its closest other living group, and which group that is.
fn closest_to(groups: &[Option<Group>], i: usize) -> (f32, usize) {
    let Some(group) = groups[i].as_ref() else {
        return (f32::INFINITY, i);
    };
    groups
        .iter()
        .enumerate()
        .filter(|(k, other)| *k != i && other.is_some())
        .map(|(k, other)| (group.distance(other.as_ref().expect("filtered")), k))
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .unwrap_or((f32::INFINITY, i))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece(window: usize, angle: f32) -> Piece {
        Piece {
            window,
            local: (angle * 10.0) as usize,
            print: vec![angle.cos(), angle.sin()],
        }
    }

    #[test]
    fn close_prints_join_and_far_ones_stay_apart() {
        let pieces = [piece(0, 0.0), piece(1, 3.0), piece(1, 0.1), piece(2, 3.1), piece(3, 0.05)];
        assert_eq!(cluster_voices(&pieces, None), vec![0, 1, 0, 1, 0]);
    }

    #[test]
    fn two_local_speakers_of_one_window_never_join_however_close() {
        let pieces = [piece(0, 0.0), piece(0, 0.11)];
        assert_eq!(cluster_voices(&pieces, None), vec![0, 1]);
        assert_eq!(cluster_voices(&pieces, Some(1)), vec![0, 1]);
    }

    #[test]
    fn two_runs_of_one_local_speaker_can_join_or_stay_apart() {
        let run = |angle: f32| Piece { window: 0, local: 0, print: vec![angle.cos(), angle.sin()] };
        assert_eq!(cluster_voices(&[run(0.0), run(0.05)], None), vec![0, 0]);
        assert_eq!(cluster_voices(&[run(0.0), run(1.5)], None), vec![0, 1]);
    }

    #[test]
    fn a_group_with_another_local_of_a_window_cannot_take_a_piece_of_that_window() {
        assert!(speaks_apart(&[(0, 0), (1, 1)], &[(1, 2)]));
        assert!(!speaks_apart(&[(0, 0), (1, 1)], &[(1, 1), (2, 0)]));
        assert!(!speaks_apart(&[(0, 0)], &[(1, 0)]));
    }

    #[test]
    fn a_ceiling_on_voices_joins_far_groups_too() {
        let pieces = [piece(0, 0.0), piece(1, 1.5), piece(2, 3.0)];
        assert_eq!(cluster_voices(&pieces, None), vec![0, 1, 2]);
        let joined = cluster_voices(&pieces, Some(2));
        assert_eq!(joined.iter().max(), Some(&1));
    }

    #[test]
    fn no_pieces_give_no_voices() {
        assert!(cluster_voices(&[], None).is_empty());
    }
}
