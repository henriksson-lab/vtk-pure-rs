//! Compute real spherical harmonics Y_l^m(theta, phi) on mesh points.
//!
//! Evaluates spherical harmonics for l=0,1,2 (s, p, d orbitals) and adds
//! the result as a scalar point data array.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the real spherical harmonic Y_l^m evaluated at each mesh point.
///
/// Points are converted to spherical coordinates (r, theta, phi) centered
/// at the mesh centroid, and the harmonic value is stored as a point data
/// array named "Y_{l}_{m}".
///
/// Supports l = 0, 1, 2 with corresponding m in [-l, l].
///
/// # Panics
///
/// Panics if l > 2 or |m| > l.
pub fn spherical_harmonics(input: &PolyData, l: i32, m: i32) -> PolyData {
    assert!(l >= 0 && l <= 2, "only l = 0, 1, 2 supported");
    assert!(m.abs() <= l, "|m| must be <= l");

    let n = input.points.len();
    // Compute centroid
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = input.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    if n > 0 {
        let nf = n as f64;
        cx /= nf;
        cy /= nf;
        cz /= nf;
    }

    let mut values = Vec::with_capacity(n);
    for i in 0..n {
        let p = input.points.get(i);
        let x = p[0] - cx;
        let y = p[1] - cy;
        let z = p[2] - cz;
        let r = (x * x + y * y + z * z).sqrt();

        // Spherical coordinates
        let theta = if r > 1e-15 { (z / r).acos() } else { 0.0 };
        let phi = y.atan2(x);

        let val = eval_real_ylm(l, m, theta, phi);
        values.push(val);
    }

    let name = format!("Y_{}_{}", l, m);
    let mut output = input.clone();
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&name, values, 1)));
    output
}

/// Evaluate real spherical harmonic Y_l^m(theta, phi).
///
/// Uses standard real-valued definitions:
/// - Y_l^0 uses the associated Legendre polynomial directly
/// - Y_l^m (m > 0) uses cos(m*phi) component
/// - Y_l^m (m < 0) uses sin(|m|*phi) component
fn eval_real_ylm(l: i32, m: i32, theta: f64, phi: f64) -> f64 {
    use std::f64::consts::PI;

    let cos_t = theta.cos();
    let sin_t = theta.sin();

    match (l, m) {
        // l = 0: s orbital
        (0, 0) => 0.5 * (1.0 / PI).sqrt(),

        // l = 1: p orbitals
        (1, -1) => (3.0 / (4.0 * PI)).sqrt() * sin_t * phi.sin(),
        (1, 0) => (3.0 / (4.0 * PI)).sqrt() * cos_t,
        (1, 1) => (3.0 / (4.0 * PI)).sqrt() * sin_t * phi.cos(),

        // l = 2: d orbitals
        (2, -2) => (15.0 / (16.0 * PI)).sqrt() * sin_t * sin_t * (2.0 * phi).sin(),
        (2, -1) => (15.0 / (4.0 * PI)).sqrt() * sin_t * cos_t * phi.sin(),
        (2, 0) => (5.0 / (16.0 * PI)).sqrt() * (3.0 * cos_t * cos_t - 1.0),
        (2, 1) => (15.0 / (4.0 * PI)).sqrt() * sin_t * cos_t * phi.cos(),
        (2, 2) => (15.0 / (16.0 * PI)).sqrt() * sin_t * sin_t * (2.0 * phi).cos(),

        _ => panic!("unsupported (l, m) = ({}, {})", l, m),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn y00_is_constant() {
        // Y_0^0 = 1/(2*sqrt(pi)), should be constant on all points
        let pd = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let result = spherical_harmonics(&pd, 0, 0);
        let arr = result.point_data().get_array("Y_0_0").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let v0 = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        let v1 = buf[0];
        arr.tuple_as_f64(2, &mut buf);
        let v2 = buf[0];

        let expected = 0.5 * (1.0 / std::f64::consts::PI).sqrt();
        assert!((v0 - expected).abs() < 1e-10);
        assert!((v1 - expected).abs() < 1e-10);
        assert!((v2 - expected).abs() < 1e-10);
    }

    #[test]
    fn y10_varies_with_z() {
        // Y_1^0 = sqrt(3/(4*pi)) * cos(theta), so it should vary with z
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 1.0],  // north pole relative to centroid
                [0.0, 0.0, -1.0], // south pole
                [1.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let result = spherical_harmonics(&pd, 1, 0);
        let arr = result.point_data().get_array("Y_1_0").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let v_north = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        let v_south = buf[0];

        // North and south should have opposite signs
        assert!(v_north > 0.0);
        assert!(v_south < 0.0);
    }
}
