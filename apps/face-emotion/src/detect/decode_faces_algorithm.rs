use super::face_contract::Face;
use super::prepare_detector_input_algorithm::SIDE;

/// The strides the detector answers at.
const STRIDES: [usize; 3] = [8, 16, 32];

fn output<'a>(outputs: &'a [(String, Vec<f32>)], name: &str) -> Option<&'a [f32]> {
    outputs.iter().find(|(found, _)| found == name).map(|(_, numbers)| numbers.as_slice())
}

/// The faces in the answer of the detector at or above `min_score`, in the pixels of its 640 by
/// 640 input, as OpenCV's `FaceDetectorYN` decodes them: at each place of each stride the score is
/// the square root of the class and object scores (each held to 0 to 1), the box middle is the
/// place plus its offset, the size is the exponent of its value, both times the stride, and each
/// point is the place plus its offset times the stride. `None` when an output is missing or
/// has the wrong size.
pub fn decode_faces(outputs: &[(String, Vec<f32>)], min_score: f32) -> Option<Vec<Face>> {
    let mut faces = Vec::new();
    for stride in STRIDES {
        let columns = SIDE / stride;
        let places = columns * columns;
        let classes = output(outputs, &format!("cls_{stride}"))?;
        let objects = output(outputs, &format!("obj_{stride}"))?;
        let boxes = output(outputs, &format!("bbox_{stride}"))?;
        let points = output(outputs, &format!("kps_{stride}"))?;
        if classes.len() != places || objects.len() != places || boxes.len() != 4 * places || points.len() != 10 * places {
            return None;
        }
        let step = stride as f32;
        for place in 0..places {
            let score = (classes[place].clamp(0.0, 1.0) * objects[place].clamp(0.0, 1.0)).sqrt();
            if score < min_score {
                continue;
            }
            let (row, column) = ((place / columns) as f32, (place % columns) as f32);
            let found = &boxes[place * 4..place * 4 + 4];
            let (middle_x, middle_y) = ((column + found[0]) * step, (row + found[1]) * step);
            let (width, height) = (found[2].exp() * step, found[3].exp() * step);
            let marks = &points[place * 10..place * 10 + 10];
            faces.push(Face {
                x: middle_x - width / 2.0,
                y: middle_y - height / 2.0,
                width,
                height,
                score,
                points: std::array::from_fn(|n| ((marks[2 * n] + column) * step, (marks[2 * n + 1] + row) * step)),
            });
        }
    }
    Some(faces)
}

/// The faces left when, of two whose boxes overlap more than `max_overlap`, only the surer stays.
/// The surest come first.
pub fn non_max_suppression(mut faces: Vec<Face>, max_overlap: f32) -> Vec<Face> {
    faces.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut kept: Vec<Face> = Vec::new();
    for face in faces {
        if kept.iter().all(|other| other.overlap(&face) <= max_overlap) {
            kept.push(face);
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An answer with one face at place (row 2, column 3) of stride 32.
    fn answer() -> Vec<(String, Vec<f32>)> {
        let mut outputs = Vec::new();
        for stride in STRIDES {
            let places = (SIDE / stride) * (SIDE / stride);
            let mut classes = vec![0.0; places];
            let mut objects = vec![0.0; places];
            let mut boxes = vec![0.0; 4 * places];
            let mut points = vec![0.0; 10 * places];
            if stride == 32 {
                let place = 2 * (SIDE / 32) + 3;
                classes[place] = 0.81;
                objects[place] = 1.0;
                boxes[place * 4..place * 4 + 4].copy_from_slice(&[0.5, 0.25, 0.0, 1.0f32.ln()]);
                points[place * 10] = 0.1;
            }
            outputs.push((format!("cls_{stride}"), classes));
            outputs.push((format!("obj_{stride}"), objects));
            outputs.push((format!("bbox_{stride}"), boxes));
            outputs.push((format!("kps_{stride}"), points));
        }
        outputs
    }

    #[test]
    fn a_face_is_decoded_as_opencv_does() {
        let faces = decode_faces(&answer(), 0.5).unwrap();
        assert_eq!(faces.len(), 1);
        let face = faces[0];
        assert!((face.score - 0.9).abs() < 1e-6);
        assert_eq!((face.width, face.height), (32.0, 32.0));
        assert_eq!((face.x, face.y), (3.5 * 32.0 - 16.0, 2.25 * 32.0 - 16.0));
        assert!((face.points[0].0 - 3.1 * 32.0).abs() < 1e-4 && face.points[0].1 == 64.0);
        assert!(decode_faces(&answer(), 0.95).unwrap().is_empty());
    }

    #[test]
    fn a_missing_output_is_none() {
        assert!(decode_faces(&answer()[1..], 0.5).is_none());
    }

    #[test]
    fn of_two_overlapping_faces_the_surer_stays() {
        let face = |x: f32, score: f32| Face { x, y: 0.0, width: 10.0, height: 10.0, score, points: [(0.0, 0.0); 5] };
        let kept = non_max_suppression(vec![face(0.0, 0.7), face(1.0, 0.9), face(50.0, 0.8)], 0.3);
        assert_eq!(kept.iter().map(|f| f.x).collect::<Vec<_>>(), vec![1.0, 50.0]);
    }
}
