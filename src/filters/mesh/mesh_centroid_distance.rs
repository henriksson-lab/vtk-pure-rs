//! Compute Euclidean distance from each vertex to the mesh centroid.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn centroid_distance(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    cx /= n as f64;
    cy /= n as f64;
    cz /= n as f64;
    let dists: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            ((p[0] - cx).powi(2) + (p[1] - cy).powi(2) + (p[2] - cz).powi(2)).sqrt()
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CentroidDistance",
            dists,
            1,
        )));
    result
        .point_data_mut()
        .set_active_scalars("CentroidDistance");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_centroid_dist() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = centroid_distance(&mesh);
        let arr = r.point_data().get_array("CentroidDistance").unwrap();
        // Centroid is at (1,1,0), distance from (0,0,0) is sqrt(2)
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 2.0f64.sqrt()).abs() < 0.1);
    }
}
