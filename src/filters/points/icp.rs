use crate::data::{CellLocator, PolyData};

/// Result of ICP (Iterative Closest Point) registration.
#[derive(Debug, Clone)]
pub struct IcpResult {
    /// The 4×4 transformation matrix (row-major) that best aligns source to target.
    pub transform: [[f64; 4]; 4],
    /// Root mean square error of the final alignment.
    pub rms_error: f64,
    /// Number of iterations performed.
    pub iterations: usize,
}

/// Align source PolyData to target using Iterative Closest Point (ICP).
///
/// Uses a simple point-to-point ICP with SVD-based rigid body estimation.
/// Returns the 4×4 transformation matrix that aligns source to target.
pub fn icp(
    source: &PolyData,
    target: &PolyData,
    max_iterations: usize,
    tolerance: f64,
) -> IcpResult {
    let n = source.points.len();
    if n == 0 || target.points.is_empty() {
        return IcpResult {
            transform: identity_4x4(),
            rms_error: 0.0,
            iterations: 0,
        };
    }
    if max_iterations == 0 {
        let source_pts: Vec<[f64; 3]> = (0..n).map(|i| source.points.get(i)).collect();
        let target_pts: Vec<[f64; 3]> = (0..target.points.len())
            .map(|i| target.points.get(i))
            .collect();
        let target_locator = (target.total_cells() > 0).then(|| CellLocator::build(target));
        return IcpResult {
            transform: identity_4x4(),
            rms_error: final_rms(&source_pts, &target_pts, target_locator.as_ref()),
            iterations: 0,
        };
    }

    let target_pts: Vec<[f64; 3]> = (0..target.points.len())
        .map(|i| target.points.get(i))
        .collect();
    let target_locator = (target.total_cells() > 0).then(|| CellLocator::build(target));

    let mut points1: Vec<[f64; 3]> = (0..n).map(|i| source.points.get(i)).collect();
    let mut total_transform = identity_4x4();
    let mut iterations = 0;

    for iter in 0..max_iterations {
        // Fill points with the closest points to each vertex in input.
        let closestp: Vec<[f64; 3]> = points1
            .iter()
            .map(|sp| closest_target_point(sp, &target_pts, target_locator.as_ref()))
            .collect();

        // Build the landmark transform.
        let (r, t) = landmark_transform(&points1, &closestp);

        // Accumulate transform
        let step = [
            [r[0][0], r[0][1], r[0][2], t[0]],
            [r[1][0], r[1][1], r[1][2], t[1]],
            [r[2][0], r[2][1], r[2][2], t[2]],
            [0.0, 0.0, 0.0, 1.0],
        ];
        total_transform = mul_4x4(&step, &total_transform);

        // Move mesh and compute mean point motion, equivalent to VTK's
        // RMS mean-distance check when enabled.
        let mut points2 = Vec::with_capacity(n);
        let mut totaldist = 0.0;
        for p in &points1 {
            let p2 = transform_point(&r, &t, *p);
            totaldist += distance2(*p, p2);
            points2.push(p2);
        }
        let mean_distance = (totaldist / n as f64).sqrt();
        points1 = points2;
        iterations = iter + 1;

        if mean_distance <= tolerance {
            break;
        }
    }

    IcpResult {
        transform: total_transform,
        rms_error: final_rms(&points1, &target_pts, target_locator.as_ref()),
        iterations,
    }
}

fn closest_target_point(
    query: &[f64; 3],
    points: &[[f64; 3]],
    locator: Option<&CellLocator>,
) -> [f64; 3] {
    if let Some((_, point, _)) = locator.and_then(|loc| loc.find_closest_cell(*query)) {
        point
    } else {
        nearest_point(query, points)
    }
}

fn nearest_point(query: &[f64; 3], points: &[[f64; 3]]) -> [f64; 3] {
    let mut best = points[0];
    let mut best_d = f64::MAX;
    for p in points {
        let d = (query[0] - p[0]).powi(2) + (query[1] - p[1]).powi(2) + (query[2] - p[2]).powi(2);
        if d < best_d {
            best_d = d;
            best = *p;
        }
    }
    best
}

fn centroid(pts: &[[f64; 3]]) -> [f64; 3] {
    let n = pts.len() as f64;
    let mut c = [0.0; 3];
    for p in pts {
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    [c[0] / n, c[1] / n, c[2] / n]
}

fn landmark_transform(source: &[[f64; 3]], target: &[[f64; 3]]) -> ([[f64; 3]; 3], [f64; 3]) {
    let n = source.len();
    let source_centroid = centroid(source);
    let target_centroid = centroid(target);

    if n == 1 {
        return (identity_3(), sub_3(target_centroid, source_centroid));
    }

    let mut m = [[0.0; 3]; 3];
    let mut source_norm = 0.0;
    let mut target_norm = 0.0;
    for pt in 0..n {
        let s = sub_3(source[pt], source_centroid);
        let t = sub_3(target[pt], target_centroid);
        for i in 0..3 {
            for j in 0..3 {
                m[i][j] += s[i] * t[j];
            }
        }
        source_norm += dot_3(s, s);
        target_norm += dot_3(t, t);
    }

    if source_norm <= 1e-30 || target_norm <= 1e-30 {
        return (identity_3(), sub_3(target_centroid, source_centroid));
    }

    let mut q = if n == 2 {
        two_point_quaternion(source, target)
    } else {
        dominant_quaternion(&horn_matrix(m))
    };
    normalize_4(&mut q);

    let matrix = quaternion_to_matrix(q);
    let transformed_centroid = mat_vec_3(matrix, source_centroid);
    let translation = sub_3(target_centroid, transformed_centroid);
    (matrix, translation)
}

fn identity_4x4() -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn mul_4x4(a: &[[f64; 4]; 4], b: &[[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut r = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                r[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    r
}

fn identity_3() -> [[f64; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

fn transform_point(matrix: &[[f64; 3]; 3], translation: &[f64; 3], p: [f64; 3]) -> [f64; 3] {
    [
        matrix[0][0] * p[0] + matrix[0][1] * p[1] + matrix[0][2] * p[2] + translation[0],
        matrix[1][0] * p[0] + matrix[1][1] * p[1] + matrix[1][2] * p[2] + translation[1],
        matrix[2][0] * p[0] + matrix[2][1] * p[1] + matrix[2][2] * p[2] + translation[2],
    ]
}

fn final_rms(points: &[[f64; 3]], target: &[[f64; 3]], locator: Option<&CellLocator>) -> f64 {
    if points.is_empty() || target.is_empty() {
        return 0.0;
    }
    let total: f64 = points
        .iter()
        .map(|p| distance2(*p, closest_target_point(p, target, locator)))
        .sum();
    (total / points.len() as f64).sqrt()
}

fn distance2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

fn sub_3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot_3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross_3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length_3(v: [f64; 3]) -> f64 {
    dot_3(v, v).sqrt()
}

fn normalize_3(v: [f64; 3]) -> [f64; 3] {
    let len = length_3(v);
    if len <= 1e-30 {
        [1.0, 0.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn normalize_4(v: &mut [f64; 4]) {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2] + v[3] * v[3]).sqrt();
    if len <= 1e-30 {
        *v = [1.0, 0.0, 0.0, 0.0];
    } else {
        for value in v {
            *value /= len;
        }
    }
}

fn perpendicular_3(v: [f64; 3]) -> [f64; 3] {
    let axis = if v[0].abs() < v[1].abs() {
        [0.0, -v[2], v[1]]
    } else {
        [-v[2], 0.0, v[0]]
    };
    normalize_3(axis)
}

fn mat_vec_3(m: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

fn horn_matrix(m: [[f64; 3]; 3]) -> [[f64; 4]; 4] {
    [
        [
            m[0][0] + m[1][1] + m[2][2],
            m[1][2] - m[2][1],
            m[2][0] - m[0][2],
            m[0][1] - m[1][0],
        ],
        [
            m[1][2] - m[2][1],
            m[0][0] - m[1][1] - m[2][2],
            m[0][1] + m[1][0],
            m[2][0] + m[0][2],
        ],
        [
            m[2][0] - m[0][2],
            m[0][1] + m[1][0],
            -m[0][0] + m[1][1] - m[2][2],
            m[1][2] + m[2][1],
        ],
        [
            m[0][1] - m[1][0],
            m[2][0] + m[0][2],
            m[1][2] + m[2][1],
            -m[0][0] - m[1][1] + m[2][2],
        ],
    ]
}

fn dominant_quaternion(n: &[[f64; 4]; 4]) -> [f64; 4] {
    let (eigenvalues, eigenvectors) = jacobi_4x4(*n);
    let mut max_idx = 0;
    for i in 1..4 {
        if eigenvalues[i] > eigenvalues[max_idx] {
            max_idx = i;
        }
    }
    [
        eigenvectors[0][max_idx],
        eigenvectors[1][max_idx],
        eigenvectors[2][max_idx],
        eigenvectors[3][max_idx],
    ]
}

fn jacobi_4x4(mut a: [[f64; 4]; 4]) -> ([f64; 4], [[f64; 4]; 4]) {
    let mut v = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    for _ in 0..50 {
        let mut p = 0;
        let mut q = 1;
        let mut max = a[p][q].abs();
        for i in 0..4 {
            for j in (i + 1)..4 {
                let value = a[i][j].abs();
                if value > max {
                    max = value;
                    p = i;
                    q = j;
                }
            }
        }
        if max < 1e-14 {
            break;
        }

        let tau = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
        let t = if tau >= 0.0 {
            1.0 / (tau + (1.0 + tau * tau).sqrt())
        } else {
            -1.0 / (-tau + (1.0 + tau * tau).sqrt())
        };
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;

        for k in 0..4 {
            if k != p && k != q {
                let akp = a[k][p];
                let akq = a[k][q];
                a[k][p] = c * akp - s * akq;
                a[p][k] = a[k][p];
                a[k][q] = s * akp + c * akq;
                a[q][k] = a[k][q];
            }
        }

        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];
        a[p][p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        a[q][q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        a[p][q] = 0.0;
        a[q][p] = 0.0;

        for row in &mut v {
            let vkp = row[p];
            let vkq = row[q];
            row[p] = c * vkp - s * vkq;
            row[q] = s * vkp + c * vkq;
        }
    }

    ([a[0][0], a[1][1], a[2][2], a[3][3]], v)
}

fn two_point_quaternion(source: &[[f64; 3]], target: &[[f64; 3]]) -> [f64; 4] {
    let ds = normalize_3(sub_3(source[1], source[0]));
    let dt = normalize_3(sub_3(target[1], target[0]));
    let w = dot_3(ds, dt).clamp(-1.0, 1.0);
    let c = cross_3(ds, dt);
    let r = length_3(c);
    let theta = r.atan2(w);
    if r > 1e-30 {
        let s = (theta / 2.0).sin() / r;
        [(theta / 2.0).cos(), c[0] * s, c[1] * s, c[2] * s]
    } else if w >= 0.0 {
        [1.0, 0.0, 0.0, 0.0]
    } else {
        let axis = perpendicular_3(ds);
        [0.0, axis[0], axis[1], axis[2]]
    }
}

fn quaternion_to_matrix(q: [f64; 4]) -> [[f64; 3]; 3] {
    let [w, x, y, z] = q;
    let ww = w * w;
    let wx = w * x;
    let wy = w * y;
    let wz = w * z;
    let xx = x * x;
    let yy = y * y;
    let zz = z * z;
    let xy = x * y;
    let xz = x * z;
    let yz = y * z;

    [
        [ww + xx - yy - zz, 2.0 * (-wz + xy), 2.0 * (wy + xz)],
        [2.0 * (wz + xy), ww - xx + yy - zz, 2.0 * (-wx + yz)],
        [2.0 * (-wy + xz), 2.0 * (wx + yz), ww - xx - yy + zz],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icp_identity() {
        // Source and target are the same — should converge with identity transform
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);

        let result = icp(&pd, &pd, 10, 1e-10);
        assert!(result.rms_error < 1e-6);
    }

    #[test]
    fn icp_translation() {
        let mut source = PolyData::new();
        source.points.push([0.0, 0.0, 0.0]);
        source.points.push([1.0, 0.0, 0.0]);
        source.points.push([0.0, 1.0, 0.0]);
        source.points.push([1.0, 1.0, 0.0]);

        let mut target = PolyData::new();
        target.points.push([0.1, 0.0, 0.0]);
        target.points.push([1.1, 0.0, 0.0]);
        target.points.push([0.1, 1.0, 0.0]);
        target.points.push([1.1, 1.0, 0.0]);

        let result = icp(&source, &target, 50, 1e-10);
        assert!(result.rms_error < 0.1, "rms = {}", result.rms_error);
        // Translation should be approximately [0.1, 0, 0]
        assert!(
            (result.transform[0][3] - 0.1).abs() < 0.05,
            "tx = {}",
            result.transform[0][3]
        );
    }
}
