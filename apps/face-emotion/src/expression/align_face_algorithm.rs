/// The side of the aligned face the expression model takes.
pub const FACE_SIDE: usize = 112;

/// Where the five points of a face go in the aligned 112 by 112 face (the reference points of the
/// OpenCV model zoo's expression demo).
const REFERENCE: [(f32, f32); 5] = [
    (38.2946, 51.6963),
    (73.5318, 51.5014),
    (56.0252, 71.7366),
    (41.5493, 92.3655),
    (70.7299, 92.2041),
];

/// The similarity (turn, uniform scale and shift, no mirror) that takes `from` closest to `to` in
/// the least-squares sense, as `[a, b, tx, ty]` for `u = a x - b y + tx`, `v = b x + a y + ty`.
pub fn similarity(from: &[(f32, f32); 5], to: &[(f32, f32); 5]) -> [f32; 4] {
    let mean = |points: &[(f32, f32); 5]| {
        let (sx, sy) = points.iter().fold((0.0, 0.0), |(sx, sy), (x, y)| (sx + x, sy + y));
        (sx / 5.0, sy / 5.0)
    };
    let ((mx, my), (mu, mv)) = (mean(from), mean(to));
    let (mut dot, mut cross, mut norm) = (0.0f32, 0.0f32, 0.0f32);
    for ((x, y), (u, v)) in from.iter().zip(to) {
        let (x, y, u, v) = (x - mx, y - my, u - mu, v - mv);
        dot += x * u + y * v;
        cross += x * v - y * u;
        norm += x * x + y * y;
    }
    if norm == 0.0 {
        return [1.0, 0.0, mu - mx, mv - my];
    }
    let (a, b) = (dot / norm, cross / norm);
    [a, b, mu - a * mx + b * my, mv - b * mx - a * my]
}

/// The face with these five `points` in a colour picture (`rgb`, `width` by `height`), turned and
/// scaled so its points sit on the reference points, as the input of the expression model: red,
/// green and blue planes of 112 by 112, each value scaled from 0..255 to -1..1. Pixels outside the
/// picture are black; the picture is sampled bilinearly, as OpenCV's `warpAffine` does.
pub fn align_face(rgb: &[u8], width: usize, height: usize, points: &[(f32, f32); 5]) -> Vec<f32> {
    let [a, b, tx, ty] = similarity(points, &REFERENCE);
    let scale = a * a + b * b;
    let plane = FACE_SIDE * FACE_SIDE;
    let mut input = vec![0.0f32; 3 * plane];
    let at = |x: i64, y: i64, channel: usize| -> f32 {
        if x < 0 || y < 0 || x >= width as i64 || y >= height as i64 {
            return 0.0;
        }
        f32::from(rgb[(y as usize * width + x as usize) * 3 + channel])
    };
    for v in 0..FACE_SIDE {
        for u in 0..FACE_SIDE {
            let (du, dv) = (u as f32 - tx, v as f32 - ty);
            let (x, y) = ((a * du + b * dv) / scale, (a * dv - b * du) / scale);
            let (x0, y0) = (x.floor(), y.floor());
            let (wx, wy) = (x - x0, y - y0);
            let (x0, y0) = (x0 as i64, y0 as i64);
            for channel in 0..3 {
                let value = at(x0, y0, channel) * (1.0 - wx) * (1.0 - wy)
                    + at(x0 + 1, y0, channel) * wx * (1.0 - wy)
                    + at(x0, y0 + 1, channel) * (1.0 - wx) * wy
                    + at(x0 + 1, y0 + 1, channel) * wx * wy;
                input[channel * plane + v * FACE_SIDE + u] = (value / 255.0 - 0.5) / 0.5;
            }
        }
    }
    input
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_similarity_of_shifted_and_scaled_points_is_found_exactly() {
        let from: [(f32, f32); 5] = REFERENCE.map(|(x, y)| (x * 2.0 + 10.0, y * 2.0 + 20.0));
        let [a, b, tx, ty] = similarity(&from, &REFERENCE);
        assert!((a - 0.5).abs() < 1e-5 && b.abs() < 1e-5, "{a} {b}");
        assert!((tx + 5.0).abs() < 1e-3 && (ty + 10.0).abs() < 1e-3, "{tx} {ty}");
    }

    #[test]
    fn a_face_already_in_place_is_copied_and_the_outside_is_black() {
        let (width, height) = (112, 112);
        let mut rgb = vec![0u8; width * height * 3];
        for (index, value) in rgb.iter_mut().enumerate() {
            *value = (index % 256) as u8;
        }
        let input = align_face(&rgb, width, height, &REFERENCE);
        let plane = FACE_SIDE * FACE_SIDE;
        let expected = |x: usize, y: usize, channel: usize| (f32::from(rgb[(y * width + x) * 3 + channel]) / 255.0 - 0.5) / 0.5;
        assert!((input[10 * FACE_SIDE + 20] - expected(20, 10, 0)).abs() < 1e-3);
        assert!((input[plane + 50 * FACE_SIDE + 60] - expected(60, 50, 1)).abs() < 1e-3);
        let small = align_face(&rgb[..3], 1, 1, &REFERENCE.map(|(x, y)| (x / 50.0, y / 50.0)));
        assert_eq!(small[0], -1.0);
    }
}
