//! Principal axes computation: PCA-based orientation and alignment.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Compute the principal axes of a mesh via PCA on vertex positions.
///
/// Returns (centroid, axes[3], eigenvalues[3]) sorted descending.
pub fn principal_axes(mesh: &PolyData) -> ([f64; 3], [[f64; 3]; 3], [f64; 3]) {
    let n = mesh.points.len();
    if n < 2 {
        return (
            [0.0; 3],
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            [0.0; 3],
        );
    }

    let mut c = [0.0; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            c[j] += p[j];
        }
    }
    for j in 0..3 {
        c[j] /= n as f64;
    }

    let mut cov = [[0.0; 3]; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
        for r in 0..3 {
            for cc in 0..3 {
                cov[r][cc] += d[r] * d[cc];
            }
        }
    }
    for r in 0..3 {
        for cc in 0..3 {
            cov[r][cc] /= n as f64;
        }
    }

    let (evals, evecs) = eigen_3x3(&cov);
    (c, evecs, evals)
}

/// Align mesh to its principal axes (so longest axis is X).
pub fn align_to_principal_axes(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let (centroid, axes, _) = principal_axes(mesh);
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = [p[0] - centroid[0], p[1] - centroid[1], p[2] - centroid[2]];
        pts.push([
            d[0] * axes[0][0] + d[1] * axes[0][1] + d[2] * axes[0][2],
            d[0] * axes[1][0] + d[1] * axes[1][1] + d[2] * axes[1][2],
            d[0] * axes[2][0] + d[1] * axes[2][1] + d[2] * axes[2][2],
        ]);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
}

/// Compute oriented bounding box dimensions along principal axes.
pub fn obb_dimensions(mesh: &PolyData) -> [f64; 3] {
    let n = mesh.points.len();
    if n == 0 {
        return [0.0; 3];
    }
    let (centroid, axes, _) = principal_axes(mesh);
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = [p[0] - centroid[0], p[1] - centroid[1], p[2] - centroid[2]];
        for ax in 0..3 {
            let proj = d[0] * axes[ax][0] + d[1] * axes[ax][1] + d[2] * axes[ax][2];
            min[ax] = min[ax].min(proj);
            max[ax] = max[ax].max(proj);
        }
    }
    [max[0] - min[0], max[1] - min[1], max[2] - min[2]]
}

fn eigen_3x3(m: &[[f64; 3]; 3]) -> ([f64; 3], [[f64; 3]; 3]) {
    // Power iteration for top 2 eigenvectors, cross for 3rd
    let mut v1 = [1.0, 0.0, 0.0];
    for _ in 0..30 {
        let w = mv(m, v1);
        let l = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
        if l > 1e-15 {
            v1 = [w[0] / l, w[1] / l, w[2] / l];
        }
    }
    let e1 = {
        let w = mv(m, v1);
        w[0] * v1[0] + w[1] * v1[1] + w[2] * v1[2]
    };

    let mut m2 = *m;
    for r in 0..3 {
        for c in 0..3 {
            m2[r][c] -= e1 * v1[r] * v1[c];
        }
    }
    let mut v2 = [0.0, 1.0, 0.0];
    for _ in 0..30 {
        let w = mv(&m2, v2);
        let l = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt();
        if l > 1e-15 {
            v2 = [w[0] / l, w[1] / l, w[2] / l];
        }
    }
    let e2 = {
        let w = mv(&m2, v2);
        w[0] * v2[0] + w[1] * v2[1] + w[2] * v2[2]
    };

    let v3 = [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ];
    let e3 = {
        let w = mv(m, v3);
        w[0] * v3[0] + w[1] * v3[1] + w[2] * v3[2]
    };

    ([e1, e2, e3], [v1, v2, v3])
}

fn mv(m: &[[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn axes() {
        let mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [10.0, 1.0, 0.0],
        ]);
        let (c, ax, ev) = principal_axes(&mesh);
        assert!(ev[0] > ev[1]); // X axis should be principal
        assert!((c[0] - 5.0).abs() < 0.01);
    }
    #[test]
    fn align() {
        let mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [0.0, 10.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 10.0, 0.0],
        ]);
        let result = align_to_principal_axes(&mesh);
        // After alignment, points should be centered and rotated
        assert_eq!(result.points.len(), 4);
        // Verify the mesh is centered near origin
        let mut cx = 0.0;
        for i in 0..4 {
            cx += result.points.get(i)[0];
        }
        assert!((cx / 4.0).abs() < 0.1);
    }
    #[test]
    fn obb() {
        let mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [10.0, 2.0, 0.0],
        ]);
        let dims = obb_dimensions(&mesh);
        assert!(dims[0] > dims[1]); // longest > second
    }
}
