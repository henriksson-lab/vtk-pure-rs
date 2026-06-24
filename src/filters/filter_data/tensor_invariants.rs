//! Compute principal invariants and yield criteria from symmetric tensor fields.
//!
//! Given a 3x3 symmetric stress/strain tensor stored as 6-component tuples
//! (xx, yy, zz, xy, yz, xz), computes:
//! - Principal invariants I1, I2, I3
//! - Deviatoric invariants J2, J3
//! - Von Mises equivalent stress
//! - Tresca equivalent stress
//! - Principal stresses (eigenvalues)

use crate::data::{AnyDataArray, DataArray, DataSetAttributes};

/// Compute principal invariants (I1, I2, I3) from a 6-component symmetric tensor array.
///
/// Input: 6-component tuples (xx, yy, zz, xy, yz, xz).
/// Output: 3-component tuples (I1, I2, I3).
pub fn tensor_principal_invariants(tensor: &DataArray<f64>) -> DataArray<f64> {
    assert_eq!(tensor.num_components(), 6, "tensor must have 6 components");
    let n = tensor.num_tuples();
    let mut data = Vec::with_capacity(n * 3);

    for i in 0..n {
        let t = tensor.tuple(i);
        let (xx, yy, zz, xy, yz, xz) = (t[0], t[1], t[2], t[3], t[4], t[5]);

        // I1 = trace
        let i1 = xx + yy + zz;
        // I2 = sum of principal minors
        let i2 = xx * yy + yy * zz + zz * xx - xy * xy - yz * yz - xz * xz;
        // I3 = determinant
        let i3 = xx * (yy * zz - yz * yz) - xy * (xy * zz - yz * xz) + xz * (xy * yz - yy * xz);

        data.push(i1);
        data.push(i2);
        data.push(i3);
    }

    DataArray::from_vec("PrincipalInvariants", data, 3)
}

/// Compute deviatoric invariants (J2, J3) from a 6-component symmetric tensor.
///
/// J2 = (1/2) * s_ij * s_ij where s is the deviatoric tensor.
/// J3 = det(s).
pub fn tensor_deviatoric_invariants(tensor: &DataArray<f64>) -> DataArray<f64> {
    assert_eq!(tensor.num_components(), 6);
    let n = tensor.num_tuples();
    let mut data = Vec::with_capacity(n * 2);

    for i in 0..n {
        let t = tensor.tuple(i);
        let (xx, yy, zz, xy, yz, xz) = (t[0], t[1], t[2], t[3], t[4], t[5]);

        let mean = (xx + yy + zz) / 3.0;
        let sxx = xx - mean;
        let syy = yy - mean;
        let szz = zz - mean;

        // J2 = (1/2) * (sxx^2 + syy^2 + szz^2 + 2*(xy^2 + yz^2 + xz^2))
        let j2 = 0.5 * (sxx * sxx + syy * syy + szz * szz) + xy * xy + yz * yz + xz * xz;

        // J3 = det(deviatoric)
        let j3 =
            sxx * (syy * szz - yz * yz) - xy * (xy * szz - yz * xz) + xz * (xy * yz - syy * xz);

        data.push(j2);
        data.push(j3);
    }

    DataArray::from_vec("DeviatoricInvariants", data, 2)
}

/// Compute von Mises equivalent stress from a 6-component symmetric tensor.
///
/// σ_vm = √(3 * J2) = √(0.5 * ((σxx-σyy)² + (σyy-σzz)² + (σzz-σxx)² + 6*(σxy² + σyz² + σxz²)))
pub fn von_mises_stress(tensor: &DataArray<f64>) -> DataArray<f64> {
    assert_eq!(tensor.num_components(), 6);
    let n = tensor.num_tuples();
    let mut data = Vec::with_capacity(n);

    for i in 0..n {
        let t = tensor.tuple(i);
        let (xx, yy, zz, xy, yz, xz) = (t[0], t[1], t[2], t[3], t[4], t[5]);

        let vm = (0.5 * ((xx - yy).powi(2) + (yy - zz).powi(2) + (zz - xx).powi(2))
            + 3.0 * (xy * xy + yz * yz + xz * xz))
            .sqrt();

        data.push(vm);
    }

    DataArray::from_vec("VonMisesStress", data, 1)
}

/// Compute Tresca equivalent stress (max shear stress criterion).
///
/// σ_tresca = max(|σ1-σ2|, |σ2-σ3|, |σ3-σ1|) where σ1,σ2,σ3 are principal stresses.
pub fn tresca_stress(tensor: &DataArray<f64>) -> DataArray<f64> {
    assert_eq!(tensor.num_components(), 6);
    let n = tensor.num_tuples();
    let mut data = Vec::with_capacity(n);

    for i in 0..n {
        let principals = compute_principal_stresses_single(tensor.tuple(i));
        let tresca = (principals[0] - principals[1])
            .abs()
            .max((principals[1] - principals[2]).abs())
            .max((principals[2] - principals[0]).abs());
        data.push(tresca);
    }

    DataArray::from_vec("TrescaStress", data, 1)
}

/// Compute principal stresses (eigenvalues of the symmetric tensor) as a 3-component array.
pub fn principal_stresses(tensor: &DataArray<f64>) -> DataArray<f64> {
    assert_eq!(tensor.num_components(), 6);
    let n = tensor.num_tuples();
    let mut data = Vec::with_capacity(n * 3);

    for i in 0..n {
        let p = compute_principal_stresses_single(tensor.tuple(i));
        data.push(p[0]);
        data.push(p[1]);
        data.push(p[2]);
    }

    DataArray::from_vec("PrincipalStresses", data, 3)
}

/// Compute eigenvalues of a 3x3 symmetric matrix.
///
/// Uses the closed-form solution for symmetric 3×3 eigenvalues
/// (Smith's algorithm, numerically stable for real symmetric matrices).
fn compute_principal_stresses_single(t: &[f64]) -> [f64; 3] {
    let (a11, a22, a33, a12, a23, a13) = (t[0], t[1], t[2], t[3], t[4], t[5]);

    let p1 = a12 * a12 + a13 * a13 + a23 * a23;

    if p1.abs() < 1e-30 {
        // Already diagonal
        let mut vals = [a11, a22, a33];
        vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        return vals;
    }

    let q = (a11 + a22 + a33) / 3.0;
    let p2 = (a11 - q).powi(2) + (a22 - q).powi(2) + (a33 - q).powi(2) + 2.0 * p1;
    let p = (p2 / 6.0).sqrt();

    // B = (1/p) * (A - q*I)
    let b11 = (a11 - q) / p;
    let b22 = (a22 - q) / p;
    let b33 = (a33 - q) / p;
    let b12 = a12 / p;
    let b13 = a13 / p;
    let b23 = a23 / p;

    // det(B) / 2
    let det_b = b11 * (b22 * b33 - b23 * b23) - b12 * (b12 * b33 - b23 * b13)
        + b13 * (b12 * b23 - b22 * b13);
    let r = (det_b / 2.0).clamp(-1.0, 1.0);

    let phi = if r <= -1.0 {
        std::f64::consts::PI / 3.0
    } else if r >= 1.0 {
        0.0
    } else {
        r.acos() / 3.0
    };

    let eig1 = q + 2.0 * p * phi.cos();
    let eig3 = q + 2.0 * p * (phi + 2.0 * std::f64::consts::PI / 3.0).cos();
    let eig2 = 3.0 * q - eig1 - eig3; // trace identity

    let mut vals = [eig1, eig2, eig3];
    vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    vals
}

/// Add all tensor analysis arrays to point data.
///
/// Input: DataSetAttributes containing a 6-component tensor array named `tensor_name`.
/// Adds: PrincipalInvariants (I1,I2,I3), DeviatoricInvariants (J2,J3),
///       VonMisesStress, TrescaStress, PrincipalStresses (σ1,σ2,σ3).
pub fn add_tensor_analysis(attrs: &mut DataSetAttributes, tensor_name: &str) {
    let tensor = if let Some(AnyDataArray::F64(arr)) = attrs.get_array(tensor_name) {
        arr.clone()
    } else {
        return;
    };
    if tensor.num_components() != 6 {
        return;
    }

    attrs.add_array(AnyDataArray::F64(tensor_principal_invariants(&tensor)));
    attrs.add_array(AnyDataArray::F64(tensor_deviatoric_invariants(&tensor)));
    attrs.add_array(AnyDataArray::F64(von_mises_stress(&tensor)));
    attrs.add_array(AnyDataArray::F64(tresca_stress(&tensor)));
    attrs.add_array(AnyDataArray::F64(principal_stresses(&tensor)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hydrostatic_tensor() -> DataArray<f64> {
        // Pure hydrostatic: σxx=σyy=σzz=100, no shear
        DataArray::from_vec("stress", vec![100.0, 100.0, 100.0, 0.0, 0.0, 0.0], 6)
    }

    fn uniaxial_tensor() -> DataArray<f64> {
        // Uniaxial tension: σxx=100, all else 0
        DataArray::from_vec("stress", vec![100.0, 0.0, 0.0, 0.0, 0.0, 0.0], 6)
    }

    #[test]
    fn hydrostatic_invariants() {
        let t = hydrostatic_tensor();
        let inv = tensor_principal_invariants(&t);
        let v = inv.tuple(0);
        assert!((v[0] - 300.0).abs() < 1e-10, "I1 should be 300");
    }

    #[test]
    fn hydrostatic_von_mises_zero() {
        let t = hydrostatic_tensor();
        let vm = von_mises_stress(&t);
        assert!(vm.tuple(0)[0].abs() < 1e-10, "hydrostatic → von Mises = 0");
    }

    #[test]
    fn uniaxial_von_mises() {
        let t = uniaxial_tensor();
        let vm = von_mises_stress(&t);
        assert!(
            (vm.tuple(0)[0] - 100.0).abs() < 1e-10,
            "uniaxial σ=100 → von Mises = 100"
        );
    }

    #[test]
    fn uniaxial_tresca() {
        let t = uniaxial_tensor();
        let tr = tresca_stress(&t);
        assert!(
            (tr.tuple(0)[0] - 100.0).abs() < 1e-10,
            "uniaxial σ=100 → Tresca = 100"
        );
    }

    #[test]
    fn uniaxial_principal_stresses() {
        let t = uniaxial_tensor();
        let ps = principal_stresses(&t);
        let v = ps.tuple(0);
        assert!(
            (v[0] - 100.0).abs() < 1e-6,
            "σ1 should be 100, got {}",
            v[0]
        );
        assert!(v[1].abs() < 1e-6, "σ2 should be 0, got {}", v[1]);
        assert!(v[2].abs() < 1e-6, "σ3 should be 0, got {}", v[2]);
    }

    #[test]
    fn deviatoric_j2_uniaxial() {
        let t = uniaxial_tensor();
        let dev = tensor_deviatoric_invariants(&t);
        let j2 = dev.tuple(0)[0];
        // J2 = σ²/3 for uniaxial
        assert!((j2 - 10000.0 / 3.0).abs() < 1e-6);
    }
}
