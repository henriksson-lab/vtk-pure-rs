//! Point Feature Histogram (PFH) for local shape description.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn point_feature_histogram(
    mesh: &PolyData,
    vertex: usize,
    k_neighbors: usize,
    angle_bins: usize,
) -> Vec<f64> {
    let n = mesh.points.len();
    if vertex >= n {
        return vec![];
    }
    let k = k_neighbors.min(n - 1).max(1);
    let ab = angle_bins.max(3);
    let p = mesh.points.get(vertex);
    // Find k nearest neighbors
    let mut dists: Vec<(usize, f64)> = (0..n)
        .filter(|&i| i != vertex)
        .map(|i| {
            let q = mesh.points.get(i);
            (
                i,
                (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2),
            )
        })
        .collect();
    dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let neighbors: Vec<usize> = dists.iter().take(k).map(|&(i, _)| i).collect();
    // Compute PFH features for each pair in neighborhood
    let nm = calc_nm(mesh);
    let mut hist = vec![0.0f64; ab * ab * ab];
    for &i in &neighbors {
        for &j in &neighbors {
            if i == j {
                continue;
            }
            let pi = mesh.points.get(i);
            let pj = mesh.points.get(j);
            let d = [pj[0] - pi[0], pj[1] - pi[1], pj[2] - pi[2]];
            let dl = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-15);
            let u = nm[i];
            let v = cross(u, normalize([d[0] / dl, d[1] / dl, d[2] / dl]));
            let w = cross(u, normalize(v));
            let alpha = dot(v, nm[j]).acos();
            let phi = dot(u, [d[0] / dl, d[1] / dl, d[2] / dl]);
            let theta = dot(w, nm[j]).atan2(dot(u, nm[j]));
            let ai = ((alpha / std::f64::consts::PI * ab as f64).floor() as usize).min(ab - 1);
            let pi2 = (((phi + 1.0) / 2.0 * ab as f64).floor() as usize).min(ab - 1);
            let ti = (((theta + std::f64::consts::PI) / (2.0 * std::f64::consts::PI) * ab as f64)
                .floor() as usize)
                .min(ab - 1);
            hist[ai * ab * ab + pi2 * ab + ti] += 1.0;
        }
    }
    let total: f64 = hist.iter().sum();
    if total > 0.0 {
        for h in &mut hist {
            *h /= total;
        }
    }
    hist
}
fn calc_nm(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &v in cell {
            let vi = v as usize;
            if vi < n {
                nm[vi][0] += fn_[0];
                nm[vi][1] += fn_[1];
                nm[vi][2] += fn_[2];
            }
        }
    }
    for v in &mut nm {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 1e-15 {
            v[0] /= l;
            v[1] /= l;
            v[2] /= l;
        }
    }
    nm
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let h = point_feature_histogram(&m, 0, 3, 3);
        assert_eq!(h.len(), 27);
    }
}
