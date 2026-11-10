use crate::detect::Face;

/// The expression of a followed face in one picture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reading {
    /// The number of the face.
    pub face: usize,
    /// When the picture is shown and how long, in milliseconds.
    pub time_ms: u64,
    /// How long the picture is shown, in milliseconds.
    pub duration_ms: u64,
    /// The place of the expression in the list of expressions.
    pub label: usize,
    /// How sure the model is of it.
    pub confidence: f32,
    /// Where the face is.
    pub place: Face,
}

/// A stretch where one face keeps one expression.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Run {
    /// The number of the face.
    pub face: usize,
    /// The place of the expression in the list of expressions.
    pub label: usize,
    /// When the first picture of the run is shown.
    pub start_ms: u64,
    /// When the last picture of the run stops being shown.
    pub end_ms: u64,
    /// The mean confidence over its pictures.
    pub confidence: f32,
    /// In how many pictures it was seen.
    pub samples: usize,
    /// Where the face was in the first picture.
    pub place: Face,
}

/// Joins the readings of each face into runs of one expression: the next reading of the same face
/// with the same expression, starting at most `gap_ms` after the run ends, continues it; anything
/// else starts a new run. Runs seen in fewer than `min_samples` pictures are dropped. Runs come in
/// the order they start, then by face.
pub fn expression_runs(readings: &[Reading], gap_ms: u64, min_samples: usize) -> Vec<Run> {
    let mut ordered: Vec<&Reading> = readings.iter().collect();
    ordered.sort_by(|a, b| a.face.cmp(&b.face).then(a.time_ms.cmp(&b.time_ms)));
    let mut runs: Vec<Run> = Vec::new();
    for reading in ordered {
        let end = reading.time_ms + reading.duration_ms;
        match runs.last_mut() {
            Some(run)
                if run.face == reading.face
                    && run.label == reading.label
                    && reading.time_ms <= run.end_ms + gap_ms =>
            {
                run.confidence = (run.confidence * run.samples as f32 + reading.confidence) / (run.samples + 1) as f32;
                run.samples += 1;
                run.end_ms = run.end_ms.max(end);
            }
            _ => runs.push(Run {
                face: reading.face,
                label: reading.label,
                start_ms: reading.time_ms,
                end_ms: end,
                confidence: reading.confidence,
                samples: 1,
                place: reading.place,
            }),
        }
    }
    runs.retain(|run| run.samples >= min_samples);
    runs.sort_by(|a, b| a.start_ms.cmp(&b.start_ms).then(a.face.cmp(&b.face)));
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(face: usize, time_ms: u64, label: usize) -> Reading {
        Reading {
            face,
            time_ms,
            duration_ms: 500,
            label,
            confidence: 0.8,
            place: Face { x: 0.0, y: 0.0, width: 1.0, height: 1.0, score: 1.0, points: [(0.0, 0.0); 5] },
        }
    }

    #[test]
    fn one_face_keeping_an_expression_is_one_run_until_it_changes() {
        let readings = [reading(0, 0, 3), reading(0, 500, 3), reading(1, 500, 4), reading(0, 1000, 0)];
        let runs = expression_runs(&readings, 1000, 1);
        let seen: Vec<(usize, usize, u64, u64, usize)> =
            runs.iter().map(|run| (run.face, run.label, run.start_ms, run.end_ms, run.samples)).collect();
        assert_eq!(seen, vec![(0, 3, 0, 1000, 2), (1, 4, 500, 1000, 1), (0, 0, 1000, 1500, 1)]);
    }

    #[test]
    fn a_long_gap_splits_a_run_and_short_runs_can_be_dropped() {
        let readings = [reading(0, 0, 3), reading(0, 5000, 3), reading(0, 5500, 3)];
        let runs = expression_runs(&readings, 1000, 2);
        assert_eq!(runs.len(), 1);
        assert_eq!((runs[0].start_ms, runs[0].samples), (5000, 2));
    }
}
