//! Estimate Laplacian eigenvalues via power iteration.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn estimate_fiedler_vector(mesh: &PolyData, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
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
    // Power iteration on Laplacian for second-smallest eigenvector
    let mut v: Vec<f64> = (0..n).map(|i| i as f64 / n as f64 - 0.5).collect();
    for _ in 0..iterations {
        let mut lv = vec![0.0f64; n];
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let deg = nb[i].len() as f64;
            lv[i] = deg * v[i];
            for &j in &nb[i] {
                lv[i] -= v[j];
            }
        }
        // Remove constant component
        let mean = lv.iter().sum::<f64>() / n as f64;
        for x in &mut lv {
            *x -= mean;
        }
        let norm = lv.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-15);
        v = lv.iter().map(|x| x / norm).collect();
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Fiedler", v, 1)));
    r.point_data_mut().set_active_scalars("Fiedler");
    r
}
pub fn spectral_partition(mesh: &PolyData, iterations: usize) -> PolyData {
    let r = estimate_fiedler_vector(mesh, iterations);
    let arr = r.point_data().get_array("Fiedler").unwrap();
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let labels: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] >= 0.0 {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    let mut result = r;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Partition",
            labels,
            1,
        )));
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fiedler() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = estimate_fiedler_vector(&m, 20);
        assert!(r.point_data().get_array("Fiedler").is_some());
    }
    #[test]
    fn test_partition() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = spectral_partition(&m, 20);
        assert!(r.point_data().get_array("Partition").is_some());
    }
}
