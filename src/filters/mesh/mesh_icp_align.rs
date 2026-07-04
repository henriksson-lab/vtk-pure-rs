//! Simple ICP (Iterative Closest Point) alignment.
use crate::data::{CellLocator, PolyData};

pub fn icp_align(source: &PolyData, target: &PolyData, max_iterations: usize) -> PolyData {
    let sn = source.points.len();
    let tn = target.points.len();
    if sn == 0 || tn == 0 {
        return source.clone();
    }

    let tpts: Vec<[f64; 3]> = (0..tn).map(|i| target.points.get(i)).collect();
    let locator = (target.total_cells() > 0).then(|| CellLocator::build(target));
    let mut points1: Vec<[f64; 3]> = (0..sn).map(|i| source.points.get(i)).collect();

    for _ in 0..max_iterations {
        let closestp: Vec<[f64; 3]> = points1
            .iter()
            .map(|p| closest_target_point(p, &tpts, locator.as_ref()))
            .collect();
        let (matrix, translation) = landmark_transform(&points1, &closestp);

        let mut points2 = Vec::with_capacity(sn);
        for p in points1 {
            points2.push(transform_point(&matrix, &translation, p));
        }
        points1 = points2;
    }

    let mut r = source.clone();
    for (i, p) in points1.into_iter().enumerate() {
        r.points.set(i, p);
    }
    r
}

pub fn icp_error(source: &PolyData, target: &PolyData) -> f64 {
    let sn = source.points.len();
    if sn == 0 {
        return 0.0;
    }
    let tn = target.points.len();
    if tn == 0 {
        return f64::INFINITY;
    }
    let mut total = 0.0;
    for i in 0..sn {
        let p = source.points.get(i);
        let mut best = f64::INFINITY;
        for j in 0..tn {
            let q = target.points.get(j);
            let d = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
            best = best.min(d);
        }
        total += best.sqrt();
    }
    total / sn as f64
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
        let d = distance2(*query, *p);
        if d < best_d {
            best_d = d;
            best = *p;
        }
    }
    best
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
        let a = sub_3(source[pt], source_centroid);
        let b = sub_3(target[pt], target_centroid);
        for i in 0..3 {
            m[i][0] += a[i] * b[0];
            m[i][1] += a[i] * b[1];
            m[i][2] += a[i] * b[2];
        }
        source_norm += dot_3(a, a);
        target_norm += dot_3(b, b);
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

fn centroid(points: &[[f64; 3]]) -> [f64; 3] {
    let mut c = [0.0; 3];
    for p in points {
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    let n = points.len() as f64;
    [c[0] / n, c[1] / n, c[2] / n]
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
    fn test_align() {
        let src = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let tgt = PolyData::from_triangles(
            vec![[5.0, 5.0, 0.0], [6.0, 5.0, 0.0], [5.5, 6.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let aligned = icp_align(&src, &tgt, 20);
        let err_before = icp_error(&src, &tgt);
        let err_after = icp_error(&aligned, &tgt);
        assert!(err_after < err_before);
    }
    #[test]
    fn test_error() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!(icp_error(&a, &a) < 1e-10);
    }
}
