//! Compute radial distance from centroid projected onto XY plane.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn radial_distance(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
    }
    cx /= n as f64;
    cy /= n as f64;
    let dists: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt()
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "RadialDistance",
            dists,
            1,
        )));
    result.point_data_mut().set_active_scalars("RadialDistance");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_radial() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = radial_distance(&mesh);
        assert!(r.point_data().get_array("RadialDistance").is_some());
    }
}
