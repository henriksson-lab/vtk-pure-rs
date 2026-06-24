/// An implicit function that can be evaluated at any point in 3D space.
///
/// Returns a signed scalar value where negative values are "inside",
/// zero is on the surface, and positive values are "outside".
pub trait ImplicitFunction {
    fn evaluate(&self, x: f64, y: f64, z: f64) -> f64;

    fn gradient(&self, x: f64, y: f64, z: f64) -> [f64; 3] {
        // Numerical gradient by default
        let h = 1e-6;
        let fx = (self.evaluate(x + h, y, z) - self.evaluate(x - h, y, z)) / (2.0 * h);
        let fy = (self.evaluate(x, y + h, z) - self.evaluate(x, y - h, z)) / (2.0 * h);
        let fz = (self.evaluate(x, y, z + h) - self.evaluate(x, y, z - h)) / (2.0 * h);
        [fx, fy, fz]
    }
}

/// Implicit plane: `dot(p - origin, normal)`.
#[derive(Debug, Clone)]
pub struct ImplicitPlane {
    pub origin: [f64; 3],
    pub normal: [f64; 3],
}

impl ImplicitPlane {
    pub fn new(origin: [f64; 3], normal: [f64; 3]) -> Self {
        Self { origin, normal }
    }
}

impl ImplicitFunction for ImplicitPlane {
    fn evaluate(&self, x: f64, y: f64, z: f64) -> f64 {
        (x - self.origin[0]) * self.normal[0]
            + (y - self.origin[1]) * self.normal[1]
            + (z - self.origin[2]) * self.normal[2]
    }

    fn gradient(&self, _x: f64, _y: f64, _z: f64) -> [f64; 3] {
        self.normal
    }
}

/// Implicit sphere: `(x-cx)^2 + (y-cy)^2 + (z-cz)^2 - r^2`.
#[derive(Debug, Clone)]
pub struct ImplicitSphere {
    pub center: [f64; 3],
    pub radius: f64,
}

impl ImplicitSphere {
    pub fn new(center: [f64; 3], radius: f64) -> Self {
        Self { center, radius }
    }
}

impl ImplicitFunction for ImplicitSphere {
    fn evaluate(&self, x: f64, y: f64, z: f64) -> f64 {
        let dx = x - self.center[0];
        let dy = y - self.center[1];
        let dz = z - self.center[2];
        dx * dx + dy * dy + dz * dz - self.radius * self.radius
    }

    fn gradient(&self, x: f64, y: f64, z: f64) -> [f64; 3] {
        [
            2.0 * (x - self.center[0]),
            2.0 * (y - self.center[1]),
            2.0 * (z - self.center[2]),
        ]
    }
}

/// Implicit axis-aligned box: maximum of distance from each face.
#[derive(Debug, Clone)]
pub struct ImplicitBox {
    pub bounds: [f64; 6], // [x_min, x_max, y_min, y_max, z_min, z_max]
}

impl ImplicitBox {
    pub fn new(bounds: [f64; 6]) -> Self {
        Self { bounds }
    }

    pub fn from_center_size(center: [f64; 3], size: [f64; 3]) -> Self {
        Self {
            bounds: [
                center[0] - size[0] / 2.0,
                center[0] + size[0] / 2.0,
                center[1] - size[1] / 2.0,
                center[1] + size[1] / 2.0,
                center[2] - size[2] / 2.0,
                center[2] + size[2] / 2.0,
            ],
        }
    }
}

impl ImplicitFunction for ImplicitBox {
    fn evaluate(&self, x: f64, y: f64, z: f64) -> f64 {
        // Positive outside, negative inside
        let dx = (self.bounds[0] - x).max(x - self.bounds[1]).max(0.0);
        let dy = (self.bounds[2] - y).max(y - self.bounds[3]).max(0.0);
        let dz = (self.bounds[4] - z).max(z - self.bounds[5]).max(0.0);

        let outside_dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if outside_dist > 0.0 {
            return outside_dist;
        }

        // Inside: return negative of distance to nearest face
        let d = [
            x - self.bounds[0],
            self.bounds[1] - x,
            y - self.bounds[2],
            self.bounds[3] - y,
            z - self.bounds[4],
            self.bounds[5] - z,
        ];
        let min_d = d.iter().cloned().fold(f64::MAX, f64::min);
        -min_d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plane_evaluation() {
        let plane = ImplicitPlane::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert!((plane.evaluate(0.0, 0.0, 1.0) - 1.0).abs() < 1e-10);
        assert!((plane.evaluate(0.0, 0.0, -1.0) + 1.0).abs() < 1e-10);
        assert!((plane.evaluate(0.0, 0.0, 0.0)).abs() < 1e-10);
    }

    #[test]
    fn sphere_evaluation() {
        let sphere = ImplicitSphere::new([0.0, 0.0, 0.0], 1.0);
        assert!(sphere.evaluate(0.0, 0.0, 0.0) < 0.0); // inside
        assert!((sphere.evaluate(1.0, 0.0, 0.0)).abs() < 1e-10); // on surface
        assert!(sphere.evaluate(2.0, 0.0, 0.0) > 0.0); // outside
    }

    #[test]
    fn box_evaluation() {
        let b = ImplicitBox::from_center_size([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        assert!(b.evaluate(0.0, 0.0, 0.0) < 0.0); // inside
        assert!((b.evaluate(1.0, 0.0, 0.0)).abs() < 1e-10); // on face
        assert!(b.evaluate(2.0, 0.0, 0.0) > 0.0); // outside
    }

    #[test]
    fn plane_gradient() {
        let plane = ImplicitPlane::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(plane.gradient(5.0, 3.0, 1.0), [1.0, 0.0, 0.0]);
    }
}
