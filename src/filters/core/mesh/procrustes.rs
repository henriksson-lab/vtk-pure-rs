use crate::data::{Points, PolyData};

/// Procrustes analysis: align source to target by optimal translation,
/// rotation, and optional uniform scaling.
///
/// Minimizes sum of squared distances between corresponding points.
/// Requires both meshes have the same number of points.
pub fn procrustes_align(source: &PolyData, target: &PolyData, allow_scaling: bool) -> PolyData {
    let n = source.points.len();
    if n == 0 || n != target.points.len() {
        return source.clone();
    }

    let (matrix, translation) = landmark_transform(source, target, allow_scaling);

    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = source.points.get(i);
        points.push([
            matrix[0][0] * p[0] + matrix[0][1] * p[1] + matrix[0][2] * p[2] + translation[0],
            matrix[1][0] * p[0] + matrix[1][1] * p[1] + matrix[1][2] * p[2] + translation[1],
            matrix[2][0] * p[0] + matrix[2][1] * p[1] + matrix[2][2] * p[2] + translation[2],
        ]);
    }

    let mut pd = source.clone();
    pd.points = points;
    pd
}

fn landmark_transform(
    source: &PolyData,
    target: &PolyData,
    allow_scaling: bool,
) -> ([[f64; 3]; 3], [f64; 3]) {
    let n = source.points.len();
    let mut source_centroid = [0.0; 3];
    let mut target_centroid = [0.0; 3];
    for i in 0..n {
        let s = source.points.get(i);
        let t = target.points.get(i);
        for j in 0..3 {
            source_centroid[j] += s[j];
            target_centroid[j] += t[j];
        }
    }
    let nf = n as f64;
    for j in 0..3 {
        source_centroid[j] /= nf;
        target_centroid[j] /= nf;
    }

    if n == 1 {
        return (identity_3(), sub_3(target_centroid, source_centroid));
    }

    let mut m = [[0.0; 3]; 3];
    let mut source_norm = 0.0;
    let mut target_norm = 0.0;
    for pt in 0..n {
        let s = sub_3(source.points.get(pt), source_centroid);
        let t = sub_3(target.points.get(pt), target_centroid);
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

    let mut matrix = quaternion_to_matrix(q);
    if allow_scaling {
        let scale = (target_norm / source_norm).sqrt();
        for row in &mut matrix {
            for value in row {
                *value *= scale;
            }
        }
    }

    let transformed_centroid = mat_vec_3(matrix, source_centroid);
    let translation = sub_3(target_centroid, transformed_centroid);
    (matrix, translation)
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

fn two_point_quaternion(source: &PolyData, target: &PolyData) -> [f64; 4] {
    let s0 = source.points.get(0);
    let s1 = source.points.get(1);
    let t0 = target.points.get(0);
    let t1 = target.points.get(1);
    let ds = normalize_3(sub_3(s1, s0));
    let dt = normalize_3(sub_3(t1, t0));
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

fn identity_3() -> [[f64; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
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

/// Compute Procrustes distance (RMS error after alignment).
pub fn procrustes_distance(a: &PolyData, b: &PolyData) -> f64 {
    let aligned = procrustes_align(a, b, true);
    let n = aligned.points.len().min(b.points.len());
    if n == 0 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 0..n {
        let p = aligned.points.get(i);
        let q = b.points.get(i);
        sum += (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
    }
    (sum / n as f64).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_translated() {
        let mut src = PolyData::new();
        let mut tgt = PolyData::new();
        for i in 0..3 {
            src.points.push([i as f64, 0.0, 0.0]);
            tgt.points.push([i as f64 + 10.0, 0.0, 0.0]);
        }

        let result = procrustes_align(&src, &tgt, false);
        let c: f64 = (0..3).map(|i| result.points.get(i)[0]).sum::<f64>() / 3.0;
        assert!((c - 11.0).abs() < 1e-5); // centered on target
    }

    #[test]
    fn distance_identical_zero() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        assert!(procrustes_distance(&pd, &pd) < 1e-10);
    }

    #[test]
    fn scaled_alignment() {
        let mut a = PolyData::new();
        let mut b = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);
        b.points.push([0.0, 0.0, 0.0]);
        b.points.push([2.0, 0.0, 0.0]);

        let result = procrustes_align(&a, &b, true);
        let d = procrustes_distance(&result, &b);
        assert!(d < 1e-10);
    }

    #[test]
    fn rotated_alignment() {
        let mut a = PolyData::new();
        let mut b = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);
        a.points.push([0.0, 1.0, 0.0]);
        b.points.push([10.0, 20.0, 0.0]);
        b.points.push([10.0, 21.0, 0.0]);
        b.points.push([9.0, 20.0, 0.0]);

        let result = procrustes_align(&a, &b, false);
        for i in 0..3 {
            let p = result.points.get(i);
            let q = b.points.get(i);
            assert!((p[0] - q[0]).abs() < 1e-8);
            assert!((p[1] - q[1]).abs() < 1e-8);
            assert!((p[2] - q[2]).abs() < 1e-8);
        }
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(procrustes_distance(&pd, &pd), 0.0);
    }
}
