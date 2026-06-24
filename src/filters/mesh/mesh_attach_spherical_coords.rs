//! Attach spherical coordinates (r, theta, phi) as point data.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn attach_spherical_coords(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut r_data = Vec::with_capacity(n);
    let mut theta = Vec::with_capacity(n);
    let mut phi = Vec::with_capacity(n);
    for i in 0..n {
        let p = mesh.points.get(i);
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        r_data.push(r);
        theta.push(if r > 1e-15 {
            (p[2] / r).clamp(-1.0, 1.0).acos()
        } else {
            0.0
        });
        phi.push(p[1].atan2(p[0]));
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("R", r_data, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Theta", theta, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Phi", phi, 1)));
    result
}
pub fn attach_cylindrical_coords(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut rho = Vec::with_capacity(n);
    let mut phi = Vec::with_capacity(n);
    let mut z = Vec::with_capacity(n);
    for i in 0..n {
        let p = mesh.points.get(i);
        rho.push((p[0] * p[0] + p[1] * p[1]).sqrt());
        phi.push(p[1].atan2(p[0]));
        z.push(p[2]);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Rho", rho, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Phi", phi, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Z", z, 1)));
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_spherical() {
        let m = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_spherical_coords(&m);
        assert!(r.point_data().get_array("R").is_some());
        assert!(r.point_data().get_array("Theta").is_some());
    }
    #[test]
    fn test_cylindrical() {
        let m = PolyData::from_triangles(
            vec![[1.0, 0.0, 5.0], [0.0, 1.0, 3.0], [0.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_cylindrical_coords(&m);
        assert!(r.point_data().get_array("Rho").is_some());
    }
}
