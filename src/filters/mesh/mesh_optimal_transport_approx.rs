//! Approximate optimal transport (Earth Mover's Distance) between point distributions.
use crate::data::PolyData;
pub fn earth_mover_distance_1d(mesh_a: &PolyData, mesh_b: &PolyData, axis: usize) -> f64 {
    let mut va: Vec<f64> = (0..mesh_a.points.len())
        .map(|i| mesh_a.points.get(i)[axis])
        .collect();
    let mut vb: Vec<f64> = (0..mesh_b.points.len())
        .map(|i| mesh_b.points.get(i)[axis])
        .collect();
    va.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    vb.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let na = va.len();
    let nb = vb.len();
    if na == 0 || nb == 0 {
        return 0.0;
    }
    // Interpolate to same number of quantiles
    let nq = na.max(nb);
    let interp = |v: &[f64], n: usize, qi: usize| -> f64 {
        let t = qi as f64 / (nq - 1).max(1) as f64;
        let fi = t * (n - 1) as f64;
        let i0 = fi.floor() as usize;
        let i1 = (i0 + 1).min(n - 1);
        v[i0] + (v[i1] - v[i0]) * (fi - i0 as f64)
    };
    (0..nq)
        .map(|qi| (interp(&va, na, qi) - interp(&vb, nb, qi)).abs())
        .sum::<f64>()
        / nq as f64
}
pub fn chamfer_distance(mesh_a: &PolyData, mesh_b: &PolyData) -> f64 {
    let na = mesh_a.points.len();
    let nb = mesh_b.points.len();
    if na == 0 || nb == 0 {
        return f64::INFINITY;
    }
    let sum_a: f64 = (0..na)
        .map(|i| {
            let p = mesh_a.points.get(i);
            (0..nb)
                .map(|j| {
                    let q = mesh_b.points.get(j);
                    (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)
                })
                .fold(f64::INFINITY, f64::min)
                .sqrt()
        })
        .sum::<f64>()
        / na as f64;
    let sum_b: f64 = (0..nb)
        .map(|j| {
            let q = mesh_b.points.get(j);
            (0..na)
                .map(|i| {
                    let p = mesh_a.points.get(i);
                    (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)
                })
                .fold(f64::INFINITY, f64::min)
                .sqrt()
        })
        .sum::<f64>()
        / nb as f64;
    (sum_a + sum_b) / 2.0
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_emd() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[5.0, 0.0, 0.0], [6.0, 0.0, 0.0], [5.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let d = earth_mover_distance_1d(&a, &b, 0);
        assert!(d > 3.0);
    }
    #[test]
    fn test_chamfer() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let d = chamfer_distance(&a, &a);
        assert!(d < 1e-10);
    }
    #[test]
    fn test_chamfer_diff() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 5.0], [1.0, 0.0, 5.0], [0.5, 1.0, 5.0]],
            vec![[0, 1, 2]],
        );
        let d = chamfer_distance(&a, &b);
        assert!((d - 5.0).abs() < 0.5);
    }
}
