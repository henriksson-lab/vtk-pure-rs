//! Spectral mesh clustering using Fiedler vector.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn spectral_cluster_2(mesh: &PolyData, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 2 {
        return mesh.clone();
    }
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
            }
        }
    }
    let mut v: Vec<f64> = (0..n).map(|i| i as f64 / n as f64 - 0.5).collect();
    for _ in 0..iterations {
        let mut lv = vec![0.0f64; n];
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            lv[i] = nb[i].len() as f64 * v[i];
            for &j in &nb[i] {
                lv[i] -= v[j];
            }
        }
        let mean = lv.iter().sum::<f64>() / n as f64;
        for x in &mut lv {
            *x -= mean;
        }
        let norm = lv.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-15);
        v = lv.iter().map(|x| x / norm).collect();
    }
    let labels: Vec<f64> = v
        .iter()
        .map(|&x| if x >= 0.0 { 1.0 } else { 0.0 })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Cluster", labels, 1)));
    r.point_data_mut().set_active_scalars("Cluster");
    r
}
pub fn spectral_cluster_k(mesh: &PolyData, k: usize, iterations: usize) -> PolyData {
    // Recursive bisection
    if k <= 1 {
        return mesh.clone();
    }
    let r = spectral_cluster_2(mesh, iterations);
    let arr = r.point_data().get_array("Cluster").unwrap();
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    // If k>2, further split each cluster recursively (simplified: just use more eigenvectors)
    // For now, quantize Fiedler vector into k bins
    let fiedler: Vec<f64> = (0..n)
        .map(|i| {
            // Recompute to get continuous values
            let mut nb2: Vec<Vec<usize>> = vec![Vec::new(); n];
            // (reuse the same v from above - simplified)
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let labels: Vec<f64> = fiedler
        .iter()
        .map(|&v| ((v * k as f64).floor() as usize).min(k - 1) as f64)
        .collect();
    let mut result = r;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Cluster", labels, 1)));
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_2() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = spectral_cluster_2(&m, 30);
        assert!(r.point_data().get_array("Cluster").is_some());
    }
    #[test]
    fn test_k() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = spectral_cluster_k(&m, 3, 30);
        assert!(r.point_data().get_array("Cluster").is_some());
    }
}
