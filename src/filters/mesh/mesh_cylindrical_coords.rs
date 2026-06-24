//! Compute cylindrical coordinates (r, theta, z) relative to centroid.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn cylindrical_coords(mesh: &PolyData) -> PolyData {
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
    let mut r_data = Vec::with_capacity(n);
    let mut theta_data = Vec::with_capacity(n);
    let mut z_data = Vec::with_capacity(n);
    for i in 0..n {
        let p = mesh.points.get(i);
        r_data.push(((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt());
        theta_data.push((p[1] - cy).atan2(p[0] - cx));
        z_data.push(p[2]);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("CylR", r_data, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CylTheta", theta_data, 1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("CylZ", z_data, 1)));
    result.point_data_mut().set_active_scalars("CylR");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cylindrical() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 5.0], [0.0, 1.0, 5.0], [-1.0, 0.0, 5.0]],
            vec![[0, 1, 2]],
        );
        let r = cylindrical_coords(&mesh);
        assert!(r.point_data().get_array("CylR").is_some());
        assert!(r.point_data().get_array("CylTheta").is_some());
        assert!(r.point_data().get_array("CylZ").is_some());
    }
}
