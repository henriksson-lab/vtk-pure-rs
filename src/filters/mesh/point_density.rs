//! Point density estimation on meshes.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute local point density (number of neighbors within radius).
pub fn point_density(mesh: &PolyData, radius: f64) -> PolyData {
    let n = mesh.points.len();
    let r2 = radius * radius;
    let positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let density: Vec<f64> = (0..n)
        .map(|i| {
            let p = positions[i];
            let count = positions
                .iter()
                .filter(|q| {
                    let dx = p[0] - q[0];
                    let dy = p[1] - q[1];
                    let dz = p[2] - q[2];
                    dx * dx + dy * dy + dz * dz <= r2
                })
                .count();
            count as f64 - 1.0 // exclude self
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Density", density, 1,
        )));
    result.point_data_mut().set_active_scalars("Density");
    result
}

/// Compute average distance to K nearest neighbors.
pub fn avg_knn_distance(mesh: &PolyData, k: usize) -> PolyData {
    let n = mesh.points.len();
    let positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let distances: Vec<f64> = (0..n)
        .map(|i| {
            let p = positions[i];
            let mut dists: Vec<f64> = positions
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .map(|(_, q)| {
                    let dx = p[0] - q[0];
                    let dy = p[1] - q[1];
                    let dz = p[2] - q[2];
                    (dx * dx + dy * dy + dz * dz).sqrt()
                })
                .collect();
            dists.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let k = k.min(dists.len());
            if k == 0 {
                0.0
            } else {
                dists[..k].iter().sum::<f64>() / k as f64
            }
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AvgKnnDist",
            distances,
            1,
        )));
    result.point_data_mut().set_active_scalars("AvgKnnDist");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_density() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.5, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = point_density(&mesh, 2.0);
        let arr = result.point_data().get_array("Density").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 2.0); // 2 neighbors within radius 2
    }
    #[test]
    fn test_knn() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = avg_knn_distance(&mesh, 1);
        let arr = result.point_data().get_array("AvgKnnDist").unwrap();
        assert_eq!(arr.num_tuples(), 3);
    }
}
