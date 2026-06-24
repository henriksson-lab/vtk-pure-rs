//! K-means clustering on mesh scalar data.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn kmeans_scalar(mesh: &PolyData, array_name: &str, k: usize, iterations: usize) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let k = k.max(1).min(n);
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mut centers: Vec<f64> = (0..k)
        .map(|i| mn + (mx - mn) * (i as f64 + 0.5) / k as f64)
        .collect();
    let mut labels = vec![0usize; n];
    for _ in 0..iterations {
        for i in 0..n {
            let mut best = 0;
            let mut bd = f64::INFINITY;
            for (ci, &c) in centers.iter().enumerate() {
                let d = (vals[i] - c).abs();
                if d < bd {
                    bd = d;
                    best = ci;
                }
            }
            labels[i] = best;
        }
        let mut sums = vec![0.0f64; k];
        let mut counts = vec![0usize; k];
        for i in 0..n {
            sums[labels[i]] += vals[i];
            counts[labels[i]] += 1;
        }
        for ci in 0..k {
            if counts[ci] > 0 {
                centers[ci] = sums[ci] / counts[ci] as f64;
            }
        }
    }
    let data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Cluster", data, 1)));
    r.point_data_mut().set_active_scalars("Cluster");
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 0.1, 10.0, 10.1],
                1,
            )));
        let r = kmeans_scalar(&m, "s", 2, 20);
        let arr = r.point_data().get_array("Cluster").unwrap();
        let mut b1 = [0.0];
        let mut b2 = [0.0];
        arr.tuple_as_f64(0, &mut b1);
        arr.tuple_as_f64(2, &mut b2);
        assert_ne!(b1[0], b2[0]);
    }
}
