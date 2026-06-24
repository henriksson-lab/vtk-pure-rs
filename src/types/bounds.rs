/// Axis-aligned bounding box in 3D space.
///
/// # Examples
///
/// ```
/// use crate::types::BoundingBox;
///
/// let bb = BoundingBox::from_corners([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);
/// assert_eq!(bb.center(), [1.0, 1.5, 2.0]);
/// assert_eq!(bb.volume(), 24.0);
/// assert!(bb.contains([1.0, 1.0, 1.0]));
/// assert!(!bb.contains([5.0, 0.0, 0.0]));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub z_min: f64,
    pub z_max: f64,
}

impl BoundingBox {
    /// An empty (inverted) bounding box that will expand on the first point added.
    pub fn empty() -> Self {
        Self {
            x_min: f64::INFINITY,
            x_max: f64::NEG_INFINITY,
            y_min: f64::INFINITY,
            y_max: f64::NEG_INFINITY,
            z_min: f64::INFINITY,
            z_max: f64::NEG_INFINITY,
        }
    }

    /// Returns true if no points have been added.
    pub fn is_empty(&self) -> bool {
        self.x_min > self.x_max
    }

    /// Expand the bounding box to include the given point.
    pub fn expand(&mut self, point: [f64; 3]) {
        self.x_min = self.x_min.min(point[0]);
        self.x_max = self.x_max.max(point[0]);
        self.y_min = self.y_min.min(point[1]);
        self.y_max = self.y_max.max(point[1]);
        self.z_min = self.z_min.min(point[2]);
        self.z_max = self.z_max.max(point[2]);
    }

    /// Center of the bounding box.
    pub fn center(&self) -> [f64; 3] {
        [
            (self.x_min + self.x_max) * 0.5,
            (self.y_min + self.y_max) * 0.5,
            (self.z_min + self.z_max) * 0.5,
        ]
    }

    /// Diagonal length of the bounding box.
    pub fn diagonal_length(&self) -> f64 {
        let dx = self.x_max - self.x_min;
        let dy = self.y_max - self.y_min;
        let dz = self.z_max - self.z_min;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Size along each axis [dx, dy, dz].
    pub fn size(&self) -> [f64; 3] {
        [
            self.x_max - self.x_min,
            self.y_max - self.y_min,
            self.z_max - self.z_min,
        ]
    }

    /// Volume of the bounding box.
    pub fn volume(&self) -> f64 {
        let s = self.size();
        s[0] * s[1] * s[2]
    }

    /// Check if a point is inside the bounding box.
    pub fn contains(&self, point: [f64; 3]) -> bool {
        point[0] >= self.x_min
            && point[0] <= self.x_max
            && point[1] >= self.y_min
            && point[1] <= self.y_max
            && point[2] >= self.z_min
            && point[2] <= self.z_max
    }

    /// Check if two bounding boxes overlap.
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.x_min <= other.x_max
            && self.x_max >= other.x_min
            && self.y_min <= other.y_max
            && self.y_max >= other.y_min
            && self.z_min <= other.z_max
            && self.z_max >= other.z_min
    }

    /// Compute the union of two bounding boxes.
    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            x_min: self.x_min.min(other.x_min),
            x_max: self.x_max.max(other.x_max),
            y_min: self.y_min.min(other.y_min),
            y_max: self.y_max.max(other.y_max),
            z_min: self.z_min.min(other.z_min),
            z_max: self.z_max.max(other.z_max),
        }
    }

    /// Compute the intersection of two bounding boxes (may be empty).
    pub fn intersection(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            x_min: self.x_min.max(other.x_min),
            x_max: self.x_max.min(other.x_max),
            y_min: self.y_min.max(other.y_min),
            y_max: self.y_max.min(other.y_max),
            z_min: self.z_min.max(other.z_min),
            z_max: self.z_max.min(other.z_max),
        }
    }

    /// Expand the bounding box by a uniform amount in all directions.
    pub fn pad(&self, amount: f64) -> BoundingBox {
        BoundingBox {
            x_min: self.x_min - amount,
            x_max: self.x_max + amount,
            y_min: self.y_min - amount,
            y_max: self.y_max + amount,
            z_min: self.z_min - amount,
            z_max: self.z_max + amount,
        }
    }

    /// Create from min/max corners.
    pub fn from_corners(min: [f64; 3], max: [f64; 3]) -> Self {
        Self {
            x_min: min[0],
            x_max: max[0],
            y_min: min[1],
            y_max: max[1],
            z_min: min[2],
            z_max: max[2],
        }
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bounds() {
        let bb = BoundingBox::empty();
        assert!(bb.is_empty());
    }

    #[test]
    fn expand_and_center() {
        let mut bb = BoundingBox::empty();
        bb.expand([0.0, 0.0, 0.0]);
        bb.expand([2.0, 4.0, 6.0]);
        assert!(!bb.is_empty());
        assert_eq!(bb.center(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn size_and_volume() {
        let bb = BoundingBox::from_corners([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);
        assert_eq!(bb.size(), [2.0, 3.0, 4.0]);
        assert_eq!(bb.volume(), 24.0);
    }

    #[test]
    fn contains() {
        let bb = BoundingBox::from_corners([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(bb.contains([0.5, 0.5, 0.5]));
        assert!(!bb.contains([1.5, 0.5, 0.5]));
        assert!(bb.contains([0.0, 0.0, 0.0])); // on boundary
    }

    #[test]
    fn intersects() {
        let a = BoundingBox::from_corners([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let b = BoundingBox::from_corners([1.0, 1.0, 1.0], [3.0, 3.0, 3.0]);
        let c = BoundingBox::from_corners([5.0, 5.0, 5.0], [6.0, 6.0, 6.0]);
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn union_and_intersection() {
        let a = BoundingBox::from_corners([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let b = BoundingBox::from_corners([1.0, 1.0, 1.0], [3.0, 3.0, 3.0]);

        let u = a.union(&b);
        assert_eq!(u.x_min, 0.0);
        assert_eq!(u.x_max, 3.0);

        let i = a.intersection(&b);
        assert_eq!(i.x_min, 1.0);
        assert_eq!(i.x_max, 2.0);
    }

    #[test]
    fn pad() {
        let bb = BoundingBox::from_corners([1.0, 1.0, 1.0], [2.0, 2.0, 2.0]);
        let padded = bb.pad(0.5);
        assert_eq!(padded.x_min, 0.5);
        assert_eq!(padded.x_max, 2.5);
    }
}
