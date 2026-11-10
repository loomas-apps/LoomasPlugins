use loomasplugins_picture_ops::resize_bilinear;

/// The side of the square picture the face detector takes.
pub const SIDE: usize = 640;

/// The input of the face detector for a colour picture (`rgb`, `width` by `height`): the picture
/// made smaller when a side is over 640 pixels, put in the top left of a black 640 by 640 square,
/// as blue, green and red planes of 0 to 255. Also gives the scale it was made smaller by (1 when
/// it was not), to bring the boxes back to the picture's pixels.
pub fn prepare_detector_input(rgb: &[u8], width: usize, height: usize) -> (Vec<f32>, f32) {
    let scale = (SIDE as f32 / width.max(height) as f32).min(1.0);
    let (new_width, new_height) = (
        ((width as f32 * scale).round() as usize).clamp(1, SIDE),
        ((height as f32 * scale).round() as usize).clamp(1, SIDE),
    );
    let resized;
    let picture: &[u8] = if (new_width, new_height) == (width, height) {
        rgb
    } else {
        resized = resize_bilinear(rgb, width, height, new_width, new_height);
        &resized
    };
    let plane = SIDE * SIDE;
    let mut input = vec![0.0f32; 3 * plane];
    for y in 0..new_height {
        for x in 0..new_width {
            let pixel = &picture[(y * new_width + x) * 3..][..3];
            let at = y * SIDE + x;
            input[at] = f32::from(pixel[2]);
            input[plane + at] = f32::from(pixel[1]);
            input[2 * plane + at] = f32::from(pixel[0]);
        }
    }
    (input, new_width as f32 / width as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_small_picture_goes_top_left_in_blue_green_red() {
        let rgb = [10u8, 20, 30, 40, 50, 60];
        let (input, scale) = prepare_detector_input(&rgb, 2, 1);
        assert_eq!(scale, 1.0);
        let plane = SIDE * SIDE;
        assert_eq!((input[0], input[plane], input[2 * plane]), (30.0, 20.0, 10.0));
        assert_eq!(input[1], 60.0);
        assert_eq!(input[SIDE], 0.0);
    }

    #[test]
    fn a_wide_picture_is_made_smaller_to_fit() {
        let rgb = vec![100u8; 1280 * 720 * 3];
        let (input, scale) = prepare_detector_input(&rgb, 1280, 720);
        assert!((scale - 0.5).abs() < 1e-6);
        assert_eq!(input[359 * SIDE + 639], 100.0);
        assert_eq!(input[360 * SIDE], 0.0);
    }
}
