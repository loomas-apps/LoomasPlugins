use crate::detect::Face;

/// How far the middle of a face can move between two pictures and still be the same face, in
/// widths of the face.
const MAX_MOVE: f32 = 0.75;
/// How much bigger or smaller a face can get between two pictures and still be the same face.
const MAX_GROWTH: f32 = 2.0;

/// A face seen in one picture: when, and where.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sighting {
    /// When the picture is shown, in milliseconds from the start of the source.
    pub time_ms: u64,
    /// The face.
    pub face: Face,
}

/// How far apart two faces are, in widths of the wider one, or `None` when they cannot be the same
/// face (too far, or too different in size).
fn distance(a: &Face, b: &Face) -> Option<f32> {
    let ratio = a.width.max(b.width) / a.width.min(b.width).max(1e-3);
    let ((ax, ay), (bx, by)) = (a.centre(), b.centre());
    let moved = ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt() / a.width.max(b.width).max(1e-3);
    (ratio <= MAX_GROWTH && moved <= MAX_MOVE).then_some(moved)
}

/// Follows the faces from picture to picture: each sighting gets the number of a face. A sighting
/// continues the face whose last sighting is closest to it, if that was at most `max_gap_ms` before
/// and in an earlier picture, the closest pairs first; otherwise it starts a new face. `sightings`
/// come in the order of their time; faces are numbered from 0 in the order they first show.
pub fn track_faces(sightings: &[Sighting], max_gap_ms: u64) -> Vec<usize> {
    let mut tracks: Vec<Sighting> = Vec::new();
    let mut numbers = vec![0; sightings.len()];
    let mut start = 0;
    while start < sightings.len() {
        let time = sightings[start].time_ms;
        let end = start + sightings[start..].iter().take_while(|seen| seen.time_ms == time).count();
        let mut pairs: Vec<(f32, usize, usize)> = Vec::new();
        for (index, seen) in sightings[start..end].iter().enumerate() {
            for (track, last) in tracks.iter().enumerate() {
                if last.time_ms < time && time - last.time_ms <= max_gap_ms {
                    if let Some(moved) = distance(&last.face, &seen.face) {
                        pairs.push((moved, start + index, track));
                    }
                }
            }
        }
        pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut placed = vec![false; end - start];
        let mut taken = vec![false; tracks.len()];
        for (_, index, track) in pairs {
            if !placed[index - start] && !taken[track] {
                placed[index - start] = true;
                taken[track] = true;
                numbers[index] = track;
            }
        }
        for index in start..end {
            if !placed[index - start] {
                numbers[index] = tracks.len();
                tracks.push(sightings[index]);
            } else {
                tracks[numbers[index]] = sightings[index];
            }
        }
        start = end;
    }
    numbers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seen(time_ms: u64, x: f32) -> Sighting {
        Sighting { time_ms, face: Face { x, y: 0.0, width: 40.0, height: 50.0, score: 0.9, points: [(0.0, 0.0); 5] } }
    }

    #[test]
    fn two_faces_keep_their_numbers_as_they_move_a_little() {
        let sightings = [seen(0, 0.0), seen(0, 200.0), seen(500, 210.0), seen(500, 10.0), seen(1000, 20.0)];
        assert_eq!(track_faces(&sightings, 2000), vec![0, 1, 1, 0, 0]);
    }

    #[test]
    fn a_face_gone_too_long_or_too_far_is_a_new_face() {
        let sightings = [seen(0, 0.0), seen(5000, 0.0), seen(5500, 300.0)];
        assert_eq!(track_faces(&sightings, 2000), vec![0, 1, 2]);
    }
}
