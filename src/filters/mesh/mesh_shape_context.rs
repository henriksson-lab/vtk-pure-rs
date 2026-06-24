//! Shape context descriptor for shape matching.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn shape_context_histogram(
    mesh: &PolyData,
    vertex: usize,
    r_bins: usize,
    theta_bins: usize,
) -> Vec<f64> {
    let n = mesh.points.len();
    if vertex >= n {
        return vec![];
    }
    let p = mesh.points.get(vertex);
    let rb = r_bins.max(1);
    let tb = theta_bins.max(1);
    let mut hist = vec![0.0f64; rb * tb];
    let mut max_r = 0.0f64;
    for i in 0..n {
        if i == vertex {
            continue;
        }
        let q = mesh.points.get(i);
        let d = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
        max_r = max_r.max(d);
    }
    if max_r < 1e-15 {
        return hist;
    }
    for i in 0..n {
        if i == vertex {
            continue;
        }
        let q = mesh.points.get(i);
        let dx = q[0] - p[0];
        let dy = q[1] - p[1];
        let dz = q[2] - p[2];
        let d = (dx * dx + dy * dy + dz * dz).sqrt();
        let theta = dy.atan2(dx);
        let ri = ((d / max_r * rb as f64).floor() as usize).min(rb - 1);
        let ti = (((theta + std::f64::consts::PI) / (2.0 * std::f64::consts::PI) * tb as f64)
            .floor() as usize)
            .min(tb - 1);
        hist[ri * tb + ti] += 1.0;
    }
    let total: f64 = hist.iter().sum();
    if total > 0.0 {
        for h in &mut hist {
            *h /= total;
        }
    }
    hist
}
pub fn attach_shape_context_entropy(mesh: &PolyData, r_bins: usize, theta_bins: usize) -> PolyData {
    let n = mesh.points.len();
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let hist = shape_context_histogram(mesh, i, r_bins, theta_bins);
            hist.iter()
                .filter(|&&h| h > 1e-30)
                .map(|&h| -h * h.ln())
                .sum::<f64>()
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("SCEntropy", data, 1)));
    r.point_data_mut().set_active_scalars("SCEntropy");
    r
}
pub fn shape_context_distance(hist_a: &[f64], hist_b: &[f64]) -> f64 {
    let n = hist_a.len().min(hist_b.len());
    if n == 0 {
        return 0.0;
    }
    (0..n)
        .map(|i| {
            let s = hist_a[i] + hist_b[i];
            if s > 1e-30 {
                (hist_a[i] - hist_b[i]).powi(2) / s
            } else {
                0.0
            }
        })
        .sum::<f64>()
        * 0.5
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hist() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let h = shape_context_histogram(&m, 0, 3, 4);
        assert_eq!(h.len(), 12);
    }
    #[test]
    fn test_entropy() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_shape_context_entropy(&m, 3, 4);
        assert!(r.point_data().get_array("SCEntropy").is_some());
    }
    #[test]
    fn test_dist() {
        let a = vec![0.5, 0.5, 0.0, 0.0];
        let b = vec![0.0, 0.0, 0.5, 0.5];
        let d = shape_context_distance(&a, &b);
        assert!(d > 0.0);
    }
}
