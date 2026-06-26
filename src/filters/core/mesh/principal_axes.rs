//! Principal axes computation: PCA-based orientation and alignment.

use crate::data::{Points, PolyData};

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
    let mut a = *m;
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    for _ in 0..50 {
        let mut p = 0usize;
        let mut q = 1usize;
        let mut max = a[0][1].abs();
        for &(r, c) in &[(0usize, 2usize), (1usize, 2usize)] {
            if a[r][c].abs() > max {
                max = a[r][c].abs();
                p = r;
                q = c;
            }
        }
        if max < 1e-12 {
            break;
        }

        let theta = 0.5 * (2.0 * a[p][q]).atan2(a[q][q] - a[p][p]);
        let cos = theta.cos();
        let sin = theta.sin();
        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];

        a[p][p] = cos * cos * app - 2.0 * sin * cos * apq + sin * sin * aqq;
        a[q][q] = sin * sin * app + 2.0 * sin * cos * apq + cos * cos * aqq;
        a[p][q] = 0.0;
        a[q][p] = 0.0;

        for r in 0..3 {
            if r == p || r == q {
                continue;
            }
            let arp = a[r][p];
            let arq = a[r][q];
            a[r][p] = cos * arp - sin * arq;
            a[p][r] = a[r][p];
            a[r][q] = sin * arp + cos * arq;
            a[q][r] = a[r][q];
        }

        for row in &mut v {
            let vrp = row[p];
            let vrq = row[q];
            row[p] = cos * vrp - sin * vrq;
            row[q] = sin * vrp + cos * vrq;
        }
    }

    let mut pairs = [
        (a[0][0], normalize([v[0][0], v[1][0], v[2][0]])),
        (a[1][1], normalize([v[0][1], v[1][1], v[2][1]])),
        (a[2][2], normalize([v[0][2], v[1][2], v[2][2]])),
    ];
    pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    (
        [pairs[0].0, pairs[1].0, pairs[2].0],
        [pairs[0].1, pairs[1].1, pairs[2].1],
    )
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 1e-15 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        v
    }
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
        assert!(ax[0][0].abs() > 0.99);
    }
    #[test]
    fn axes_sorted_when_y_is_principal() {
        let mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [0.0, 10.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 10.0, 0.0],
        ]);
        let (_, ax, ev) = principal_axes(&mesh);
        assert!(ev[0] > ev[1]);
        assert!(ax[0][1].abs() > 0.99);
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
