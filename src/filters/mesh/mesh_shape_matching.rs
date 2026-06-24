//! Shape matching via point cloud registration and descriptor comparison.
use crate::data::PolyData;
pub fn hausdorff_distance(a: &PolyData, b: &PolyData) -> f64 {
    let d_ab = directed_hausdorff(a, b);
    let d_ba = directed_hausdorff(b, a);
    d_ab.max(d_ba)
}
pub fn directed_hausdorff(from: &PolyData, to: &PolyData) -> f64 {
    let nf = from.points.len();
    let nt = to.points.len();
    if nf == 0 || nt == 0 {
        return f64::INFINITY;
    }
    let mut max_d = 0.0f64;
    for i in 0..nf {
        let p = from.points.get(i);
        let mut min_d = f64::INFINITY;
        for j in 0..nt {
            let q = to.points.get(j);
            let d = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
            min_d = min_d.min(d);
        }
        max_d = max_d.max(min_d.sqrt());
    }
    max_d
}
pub fn mean_surface_distance(a: &PolyData, b: &PolyData) -> f64 {
    let d_ab = directed_mean(a, b);
    let d_ba = directed_mean(b, a);
    (d_ab + d_ba) / 2.0
}
fn directed_mean(from: &PolyData, to: &PolyData) -> f64 {
    let nf = from.points.len();
    let nt = to.points.len();
    if nf == 0 || nt == 0 {
        return f64::INFINITY;
    }
    let sum: f64 = (0..nf)
        .map(|i| {
            let p = from.points.get(i);
            (0..nt)
                .map(|j| {
                    let q = to.points.get(j);
                    (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)
                })
                .fold(f64::INFINITY, f64::min)
                .sqrt()
        })
        .sum();
    sum / nf as f64
}
pub fn rms_surface_distance(a: &PolyData, b: &PolyData) -> f64 {
    let nf = a.points.len();
    let nt = b.points.len();
    if nf == 0 || nt == 0 {
        return f64::INFINITY;
    }
    let sum: f64 = (0..nf)
        .map(|i| {
            let p = a.points.get(i);
            (0..nt)
                .map(|j| {
                    let q = b.points.get(j);
                    (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)
                })
                .fold(f64::INFINITY, f64::min)
        })
        .sum();
    (sum / nf as f64).sqrt()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_identical() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!(hausdorff_distance(&m, &m) < 1e-10);
        assert!(mean_surface_distance(&m, &m) < 1e-10);
    }
    #[test]
    fn test_different() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 5.0], [1.0, 0.0, 5.0], [0.5, 1.0, 5.0]],
            vec![[0, 1, 2]],
        );
        assert!((hausdorff_distance(&a, &b) - 5.0).abs() < 0.1);
    }
    #[test]
    fn test_rms() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!(rms_surface_distance(&a, &a) < 1e-10);
    }
}
