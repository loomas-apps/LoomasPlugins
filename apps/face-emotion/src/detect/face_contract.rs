/// A face found in a picture, in the pixels of that picture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Face {
    /// The left edge of the box.
    pub x: f32,
    /// The top edge of the box.
    pub y: f32,
    /// The width of the box.
    pub width: f32,
    /// The height of the box.
    pub height: f32,
    /// How sure the detector is that it is a face, from 0 to 1.
    pub score: f32,
    /// The right eye, the left eye, the nose tip, and the right and left mouth corners, as the
    /// face sees them.
    pub points: [(f32, f32); 5],
}

impl Face {
    /// The share of the two boxes that they have in common: their intersection over their union.
    pub fn overlap(&self, other: &Face) -> f32 {
        let width = (self.x + self.width).min(other.x + other.width) - self.x.max(other.x);
        let height = (self.y + self.height).min(other.y + other.height) - self.y.max(other.y);
        if width <= 0.0 || height <= 0.0 {
            return 0.0;
        }
        let common = width * height;
        common / (self.width * self.height + other.width * other.height - common)
    }

    /// The middle of the box.
    pub fn centre(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn face(x: f32, y: f32, size: f32) -> Face {
        Face { x, y, width: size, height: size, score: 1.0, points: [(0.0, 0.0); 5] }
    }

    #[test]
    fn the_overlap_is_the_intersection_over_the_union() {
        assert_eq!(face(0.0, 0.0, 10.0).overlap(&face(0.0, 0.0, 10.0)), 1.0);
        assert!((face(0.0, 0.0, 10.0).overlap(&face(5.0, 0.0, 10.0)) - 50.0 / 150.0).abs() < 1e-6);
        assert_eq!(face(0.0, 0.0, 10.0).overlap(&face(20.0, 0.0, 10.0)), 0.0);
    }
}
