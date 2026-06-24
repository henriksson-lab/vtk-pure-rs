//! Compute distance between two meshes (Hausdorff and average).

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Distance metrics between two meshes.
pub struct MeshDistance {
    pub hausdorff: f64,
    pub mean: f64,
    pub rms: f64,
}

/// Compute distance from each vertex of mesh A to closest point on mesh B.
/// Attaches "Distance" point data to mesh A.
pub fn distance_to_mesh(mesh_a: &PolyData, mesh_b: &PolyData) -> PolyData {
    let n = mesh_a.points.len();
    let distances: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh_a.points.get(i);
            min_distance_to_surface(p, mesh_b)
        })
        .collect();
    let mut result = mesh_a.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Distance", distances, 1,
        )));
    result.point_data_mut().set_active_scalars("Distance");
    result
}

/// Compute distance metrics between two meshes.
pub fn mesh_distance_metrics(mesh_a: &PolyData, mesh_b: &PolyData) -> MeshDistance {
    let n = mesh_a.points.len();
    if n == 0 {
        return MeshDistance {
            hausdorff: 0.0,
            mean: 0.0,
            rms: 0.0,
        };
    }
    let distances: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh_a.points.get(i);
            min_distance_to_surface(p, mesh_b)
        })
        .collect();
    let hausdorff = distances.iter().cloned().fold(0.0f64, f64::max);
    let mean = distances.iter().sum::<f64>() / n as f64;
    let rms = (distances.iter().map(|d| d * d).sum::<f64>() / n as f64).sqrt();
    MeshDistance {
        hausdorff,
        mean,
        rms,
    }
}

fn min_distance_to_surface(p: [f64; 3], mesh: &PolyData) -> f64 {
    let mut best = f64::INFINITY;
    // Quick vertex distance (approximation)
    for i in 0..mesh.points.len() {
        let q = mesh.points.get(i);
        let d = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
        best = best.min(d);
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_distance() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 5.0], [1.0, 0.0, 5.0], [0.5, 1.0, 5.0]],
            vec![[0, 1, 2]],
        );
        let r = distance_to_mesh(&a, &b);
        let arr = r.point_data().get_array("Distance").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }
    #[test]
    fn test_metrics() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 3.0], [1.0, 0.0, 3.0], [0.5, 1.0, 3.0]],
            vec![[0, 1, 2]],
        );
        let m = mesh_distance_metrics(&a, &b);
        assert!((m.hausdorff - 3.0).abs() < 1e-10);
        assert!((m.mean - 3.0).abs() < 1e-10);
    }
}
