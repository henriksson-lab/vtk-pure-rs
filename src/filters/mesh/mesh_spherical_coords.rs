//! Compute spherical coordinates (r, theta, phi) relative to centroid.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn spherical_coords(mesh: &PolyData) -> PolyData {
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
    let mut r_data = Vec::with_capacity(n);
    let mut theta_data = Vec::with_capacity(n);
    let mut phi_data = Vec::with_capacity(n);
    for i in 0..n {
        let p = mesh.points.get(i);
        let dx = p[0] - cx;
        let dy = p[1] - cy;
        let dz = p[2] - cz;
        let r = (dx * dx + dy * dy + dz * dz).sqrt();
        let theta = if r > 1e-15 {
            (dz / r).clamp(-1.0, 1.0).acos()
        } else {
            0.0
        };
        let phi = dy.atan2(dx);
        r_data.push(r);
        theta_data.push(theta);
        phi_data.push(phi);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Radius", r_data, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Theta", theta_data, 1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Phi", phi_data, 1)));
    result.point_data_mut().set_active_scalars("Radius");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_spherical() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let r = spherical_coords(&mesh);
        assert!(r.point_data().get_array("Radius").is_some());
        assert!(r.point_data().get_array("Theta").is_some());
        assert!(r.point_data().get_array("Phi").is_some());
    }
}
