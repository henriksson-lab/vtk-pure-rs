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
        let p = [x, y, z];
        let min_p = [self.bounds[0], self.bounds[2], self.bounds[4]];
        let max_p = [self.bounds[1], self.bounds[3], self.bounds[5]];
        let mut min_distance = -f64::MAX;
        let mut distance = 0.0;
        let mut inside = true;

        for i in 0..3 {
            let diff = max_p[i] - min_p[i];
            let dist;
            if diff != 0.0 {
                let t = (p[i] - min_p[i]) / diff;
                if t < 0.0 {
                    inside = false;
                    dist = min_p[i] - p[i];
                } else if t > 1.0 {
                    inside = false;
                    dist = p[i] - max_p[i];
                } else {
                    dist = if t <= 0.5 {
                        min_p[i] - p[i]
                    } else {
                        p[i] - max_p[i]
                    };
                    min_distance = min_distance.max(dist);
                }
            } else {
                dist = (p[i] - min_p[i]).abs();
                if dist > 0.0 {
                    inside = false;
                }
            }

            if dist > 0.0 {
                distance += dist * dist;
            }
        }

        distance = distance.sqrt();
        if inside {
            min_distance
        } else {
            distance
        }
    }

    fn gradient(&self, x: f64, y: f64, z: f64) -> [f64; 3] {
        let point = [x, y, z];
        let min_p = [self.bounds[0], self.bounds[2], self.bounds[4]];
        let max_p = [self.bounds[1], self.bounds[3], self.bounds[5]];
        let center = [
            0.5 * (min_p[0] + max_p[0]),
            0.5 * (min_p[1] + max_p[1]),
            0.5 * (min_p[2] + max_p[2]),
        ];
        let mut loc = [0_i32; 3];
        let mut in_dir = [0.0; 3];
        let mut out_dir = [0.0; 3];
        let mut min_dist = f64::MAX;
        let mut min_axis = 0;

        for i in 0..3 {
            if point[i] < min_p[i] {
                loc[i] = 0;
                out_dir[i] = -1.0;
            } else if point[i] > max_p[i] {
                loc[i] = 2;
                out_dir[i] = 1.0;
            } else {
                loc[i] = 1;
                let dist;
                if point[i] <= center[i] {
                    dist = point[i] - min_p[i];
                    in_dir[i] = -1.0;
                } else {
                    dist = max_p[i] - point[i];
                    in_dir[i] = 1.0;
                }
                if dist < min_dist {
                    min_dist = dist;
                    min_axis = i;
                }
            }
        }

        let index = loc[0] + 3 * loc[1] + 9 * loc[2];
        match index {
            0 | 2 | 6 | 8 | 18 | 20 | 24 | 26 => normalize3([
                point[0] - center[0],
                point[1] - center[1],
                point[2] - center[2],
            ]),
            1 | 3 | 5 | 7 | 9 | 11 | 15 | 17 | 19 | 21 | 23 | 25 => {
                let mut n = [0.0; 3];
                for i in 0..3 {
                    if out_dir[i] != 0.0 {
                        n[i] = point[i] - center[i];
                    }
                }
                normalize3(n)
            }
            4 | 10 | 12 | 14 | 16 | 22 => out_dir,
            13 => {
                let mut n = [0.0; 3];
                n[min_axis] = in_dir[min_axis];
                n
            }
            _ => [0.0; 3],
        }
    }
}

fn normalize3(mut v: [f64; 3]) -> [f64; 3] {
    let norm = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if norm != 0.0 {
        v[0] /= norm;
        v[1] /= norm;
        v[2] /= norm;
    }
    v
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

    #[test]
    fn box_gradient_matches_vtk_regions() {
        let b = ImplicitBox::from_center_size([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);

        assert_eq!(b.gradient(0.0, 0.0, 0.0), [-1.0, 0.0, 0.0]);
        assert_eq!(b.gradient(2.0, 0.0, 0.0), [1.0, 0.0, 0.0]);

        let edge = b.gradient(2.0, 2.0, 0.0);
        let inv_sqrt_2 = std::f64::consts::FRAC_1_SQRT_2;
        assert!((edge[0] - inv_sqrt_2).abs() < 1e-12);
        assert!((edge[1] - inv_sqrt_2).abs() < 1e-12);
        assert_eq!(edge[2], 0.0);

        let corner = b.gradient(2.0, 2.0, 2.0);
        let inv_sqrt_3 = 1.0 / 3.0_f64.sqrt();
        assert!((corner[0] - inv_sqrt_3).abs() < 1e-12);
        assert!((corner[1] - inv_sqrt_3).abs() < 1e-12);
        assert!((corner[2] - inv_sqrt_3).abs() < 1e-12);
    }
}
