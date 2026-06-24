use crate::data::{AnyDataArray, DataArray, PolyData};

/// Add spherical coordinate arrays (r, theta, phi) as point data.
///
/// - r: distance from origin
/// - theta: polar angle from +Z (0 = north pole, pi = south pole)
/// - phi: azimuthal angle in XY plane from +X
///
/// Reference point defaults to [0,0,0].
pub fn spherical_coordinates(input: &PolyData, center: [f64; 3]) -> PolyData {
    let n = input.points.len();
    let mut r_arr = Vec::with_capacity(n);
    let mut theta_arr = Vec::with_capacity(n);
    let mut phi_arr = Vec::with_capacity(n);

    for i in 0..n {
        let p = input.points.get(i);
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        let dz = p[2] - center[2];
        let r = (dx * dx + dy * dy + dz * dz).sqrt();
        let theta = if r > 1e-15 { (dz / r).acos() } else { 0.0 };
        let phi = dy.atan2(dx);

        r_arr.push(r);
        theta_arr.push(theta);
        phi_arr.push(phi);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("R", r_arr, 1)));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Theta", theta_arr, 1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Phi", phi_arr, 1)));
    pd
}

/// Add cylindrical coordinate arrays (rho, phi, z) as point data.
///
/// - rho: distance from Z axis
/// - phi: azimuthal angle in XY plane from +X
/// - z: height
pub fn cylindrical_coordinates(input: &PolyData, center: [f64; 2]) -> PolyData {
    let n = input.points.len();
    let mut rho_arr = Vec::with_capacity(n);
    let mut phi_arr = Vec::with_capacity(n);
    let mut z_arr = Vec::with_capacity(n);

    for i in 0..n {
        let p = input.points.get(i);
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        rho_arr.push((dx * dx + dy * dy).sqrt());
        phi_arr.push(dy.atan2(dx));
        z_arr.push(p[2]);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Rho", rho_arr, 1)));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Phi", phi_arr, 1)));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("CylZ", z_arr, 1)));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spherical_on_axis() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 5.0]); // north pole at r=5

        let result = spherical_coordinates(&pd, [0.0, 0.0, 0.0]);
        let r = result.point_data().get_array("R").unwrap();
        let theta = result.point_data().get_array("Theta").unwrap();
        let mut buf = [0.0f64];
        r.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
        theta.tuple_as_f64(0, &mut buf);
        assert!(buf[0].abs() < 1e-10); // theta=0 at north pole
    }

    #[test]
    fn cylindrical_basic() {
        let mut pd = PolyData::new();
        pd.points.push([3.0, 4.0, 7.0]);

        let result = cylindrical_coordinates(&pd, [0.0, 0.0]);
        let rho = result.point_data().get_array("Rho").unwrap();
        let mut buf = [0.0f64];
        rho.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = spherical_coordinates(&pd, [0.0, 0.0, 0.0]);
        assert!(result.point_data().get_array("R").is_some());
    }
}
