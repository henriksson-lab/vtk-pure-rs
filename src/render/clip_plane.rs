/// A clipping plane that discards fragments on one side.
///
/// The plane is defined by a normal and distance: `dot(normal, point) + distance < 0` is clipped.
/// Useful for section views that reveal internal structure.
#[derive(Debug, Clone, Copy)]
pub struct ClipPlane {
    /// Plane normal (points toward the visible side).
    pub normal: [f64; 3],
    /// Signed distance from origin. Points with `dot(normal, p) + distance < 0` are clipped.
    pub distance: f64,
    /// Whether this clip plane is active.
    pub enabled: bool,
}

impl ClipPlane {
    /// Create a clip plane from a point on the plane and a normal direction.
    pub fn from_point_normal(point: [f64; 3], normal: [f64; 3]) -> Self {
        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        let n = if len > 1e-12 {
            [normal[0] / len, normal[1] / len, normal[2] / len]
        } else {
            [0.0, 0.0, 1.0]
        };
        let d = -(n[0] * point[0] + n[1] * point[1] + n[2] * point[2]);
        Self {
            normal: n,
            distance: d,
            enabled: true,
        }
    }

    /// X-axis clip plane at the given position (clips x < pos).
    pub fn x(pos: f64) -> Self {
        Self::from_point_normal([pos, 0.0, 0.0], [1.0, 0.0, 0.0])
    }

    /// Y-axis clip plane at the given position (clips y < pos).
    pub fn y(pos: f64) -> Self {
        Self::from_point_normal([0.0, pos, 0.0], [0.0, 1.0, 0.0])
    }

    /// Z-axis clip plane at the given position (clips z < pos).
    pub fn z(pos: f64) -> Self {
        Self::from_point_normal([0.0, 0.0, pos], [0.0, 0.0, 1.0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_point_normal() {
        let cp = ClipPlane::from_point_normal([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(cp.enabled);
        assert!((cp.normal[0] - 1.0).abs() < 1e-12);
        assert!((cp.distance + 1.0).abs() < 1e-12);
    }

    #[test]
    fn axis_planes() {
        let cp = ClipPlane::x(2.0);
        assert!((cp.normal[0] - 1.0).abs() < 1e-12);
        assert!((cp.distance + 2.0).abs() < 1e-12);
    }
}
