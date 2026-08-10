use crate::common::core::math::quadratic_root;

const RAY_EPSILON: f64 = 1e-12;

/// VTK: `vtkBilinearQuadIntersection`.
#[derive(Debug, Clone, PartialEq)]
pub struct BilinearQuadIntersection {
    point00: [f64; 3],
    point01: [f64; 3],
    point10: [f64; 3],
    point11: [f64; 3],
    axes_swapping: i32,
}

impl BilinearQuadIntersection {
    /// VTK: `vtkBilinearQuadIntersection::vtkBilinearQuadIntersection`.
    pub fn new(pt00: [f64; 3], pt01: [f64; 3], pt10: [f64; 3], pt11: [f64; 3]) -> Self {
        Self {
            point00: pt00,
            point01: pt01,
            point10: pt10,
            point11: pt11,
            axes_swapping: 0,
        }
    }

    /// VTK: `vtkBilinearQuadIntersection::GetP00Data`.
    pub fn get_p00_data(&mut self) -> &mut [f64; 3] {
        &mut self.point00
    }

    /// VTK: `vtkBilinearQuadIntersection::GetP01Data`.
    pub fn get_p01_data(&mut self) -> &mut [f64; 3] {
        &mut self.point01
    }

    /// VTK: `vtkBilinearQuadIntersection::GetP10Data`.
    pub fn get_p10_data(&mut self) -> &mut [f64; 3] {
        &mut self.point10
    }

    /// VTK: `vtkBilinearQuadIntersection::GetP11Data`.
    pub fn get_p11_data(&mut self) -> &mut [f64; 3] {
        &mut self.point11
    }

    /// VTK: `vtkBilinearQuadIntersection::ComputeCartesianCoordinates`.
    pub fn compute_cartesian_coordinates(&self, u: f64, v: f64) -> [f64; 3] {
        let mut respt = [
            (1.0 - u) * (1.0 - v) * self.point00[0]
                + (1.0 - u) * v * self.point01[0]
                + u * (1.0 - v) * self.point10[0]
                + u * v * self.point11[0],
            (1.0 - u) * (1.0 - v) * self.point00[1]
                + (1.0 - u) * v * self.point01[1]
                + u * (1.0 - v) * self.point10[1]
                + u * v * self.point11[1],
            (1.0 - u) * (1.0 - v) * self.point00[2]
                + (1.0 - u) * v * self.point01[2]
                + u * (1.0 - v) * self.point10[2]
                + u * v * self.point11[2],
        ];

        let mut nb_of_swap = self.axes_swapping;
        while nb_of_swap != 0 {
            let tmp = respt[2];
            respt[2] = respt[1];
            respt[1] = respt[0];
            respt[0] = tmp;
            nb_of_swap -= 1;
        }
        respt
    }

    /// VTK: `vtkBilinearQuadIntersection::RayIntersection`.
    pub fn ray_intersection(&mut self, r: [f64; 3], q: [f64; 3], uv: &mut [f64; 3]) -> bool {
        let mut qx = q[0];
        let mut qy = q[1];
        let mut qz = q[2];

        let mut rx = r[0];
        let mut ry = r[1];
        let mut rz = r[2];

        self.axes_swapping = 0;
        while qz == 0.0 && self.axes_swapping < 3 {
            self.axes_swapping += 1;
            rotate_xyz(&mut qx, &mut qy, &mut qz);
            rotate_xyz(&mut rx, &mut ry, &mut rz);
            rotate_point(&mut self.point00);
            rotate_point(&mut self.point01);
            rotate_point(&mut self.point10);
            rotate_point(&mut self.point11);
        }

        let ax = self.point11[0] - self.point10[0] - self.point01[0] + self.point00[0];
        let ay = self.point11[1] - self.point10[1] - self.point01[1] + self.point00[1];
        let az = self.point11[2] - self.point10[2] - self.point01[2] + self.point00[2];

        let bx = self.point10[0] - self.point00[0];
        let by = self.point10[1] - self.point00[1];
        let bz = self.point10[2] - self.point00[2];

        let cx = self.point01[0] - self.point00[0];
        let cy = self.point01[1] - self.point00[1];
        let cz = self.point01[2] - self.point00[2];

        let dx = self.point00[0] - rx;
        let dy = self.point00[1] - ry;
        let dz = self.point00[2] - rz;

        let a1 = ax * qz - az * qx;
        let a2 = ay * qz - az * qy;
        let b1 = bx * qz - bz * qx;
        let b2 = by * qz - bz * qy;
        let c1 = cx * qz - cz * qx;
        let c2 = cy * qz - cz * qy;
        let d1 = dx * qz - dz * qx;
        let d2 = dy * qz - dz * qy;

        let a = a2 * c1 - a1 * c2;
        let b = a2 * d1 - a1 * d2 + b2 * c1 - b1 * c2;
        let c = b2 * d1 - b1 * d2;

        uv[0] = -2.0;
        uv[1] = -2.0;
        uv[2] = -2.0;

        let (num_sol, vsol) = quadratic_root(a, b, c, -RAY_EPSILON, 1.0 + RAY_EPSILON);
        match num_sol {
            0 => false,
            1 => {
                uv[1] = vsol[0];
                uv[0] = get_best_denominator(uv[1], a2, a1, b2, b1, c2, c1, d2, d1);
                let pos1 = self.compute_cartesian_coordinates(uv[0], uv[1]);
                uv[2] = compute_intersection_factor(q, r, pos1);
                uv[0] < 1.0 + RAY_EPSILON && uv[0] > -RAY_EPSILON && uv[2] > 0.0
            }
            2 => {
                uv[1] = vsol[0];
                uv[0] = get_best_denominator(uv[1], a2, a1, b2, b1, c2, c1, d2, d1);
                let pos1 = self.compute_cartesian_coordinates(uv[0], uv[1]);
                uv[2] = compute_intersection_factor(q, r, pos1);

                if uv[0] < 1.0 + RAY_EPSILON && uv[0] > -RAY_EPSILON && uv[2] > 0.0 {
                    let u = get_best_denominator(vsol[1], a2, a1, b2, b1, c2, c1, d2, d1);
                    if u < 1.0 + RAY_EPSILON && u > RAY_EPSILON {
                        let pos2 = self.compute_cartesian_coordinates(u, vsol[1]);
                        let t2 = compute_intersection_factor(q, r, pos2);
                        if t2 < 0.0 || uv[2] < t2 {
                            return true;
                        }
                        uv[1] = vsol[1];
                        uv[0] = u;
                        uv[2] = t2;
                        return true;
                    }
                    return true;
                }

                uv[1] = vsol[1];
                uv[0] = get_best_denominator(vsol[1], a2, a1, b2, b1, c2, c1, d2, d1);
                let pos1 = self.compute_cartesian_coordinates(uv[0], uv[1]);
                uv[2] = compute_intersection_factor(q, r, pos1);
                uv[0] < 1.0 + RAY_EPSILON && uv[0] > -RAY_EPSILON && uv[2] > 0.0
            }
            _ => false,
        }
    }
}

impl Default for BilinearQuadIntersection {
    fn default() -> Self {
        Self {
            point00: [0.0; 3],
            point01: [0.0; 3],
            point10: [0.0; 3],
            point11: [0.0; 3],
            axes_swapping: 0,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn get_best_denominator(
    v: f64,
    m1: f64,
    m2: f64,
    j1: f64,
    j2: f64,
    k1: f64,
    k2: f64,
    r1: f64,
    r2: f64,
) -> f64 {
    let denom = v * (m1 - m2) + j1 - j2;
    let d2 = v * m1 + j1;
    if denom.abs() > d2.abs() {
        return (v * (k2 - k1) + r2 - r1) / denom;
    }
    -(v * k1 + r1) / d2
}

fn compute_intersection_factor(dir: [f64; 3], orig: [f64; 3], srfpos: [f64; 3]) -> f64 {
    if dir[0].abs() >= dir[1].abs() && dir[0].abs() >= dir[2].abs() {
        (srfpos[0] - orig[0]) / dir[0]
    } else if dir[1].abs() >= dir[2].abs() {
        (srfpos[1] - orig[1]) / dir[1]
    } else {
        (srfpos[2] - orig[2]) / dir[2]
    }
}

fn rotate_xyz(x: &mut f64, y: &mut f64, z: &mut f64) {
    let tmp = *x;
    *x = *y;
    *y = *z;
    *z = tmp;
}

fn rotate_point(point: &mut [f64; 3]) {
    let tmp = point[0];
    point[0] = point[1];
    point[1] = point[2];
    point[2] = tmp;
}
