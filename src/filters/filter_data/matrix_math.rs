//! Matrix math operations on data arrays.
//!
//! Supports eigenvalue decomposition, determinant, inverse, and
//! matrix multiplication for 3x3 matrices stored as 9-component tuples.

use crate::data::DataArray;

/// Compute determinant of 3x3 matrices stored as 9-component tuples.
pub fn matrix_determinant(input: &DataArray<f64>) -> DataArray<f64> {
    assert_eq!(
        input.num_components(),
        9,
        "input must have 9 components (3x3 matrix)"
    );
    let n = input.num_tuples();
    let mut data = Vec::with_capacity(n);

    for i in 0..n {
        let m = input.tuple(i);
        let det = m[0] * (m[4] * m[8] - m[5] * m[7]) - m[1] * (m[3] * m[8] - m[5] * m[6])
            + m[2] * (m[3] * m[7] - m[4] * m[6]);
        data.push(det);
    }

    DataArray::from_vec("Determinant", data, 1)
}

/// Compute inverse of 3x3 matrices stored as 9-component tuples.
/// Singular matrices produce zero matrices.
pub fn matrix_inverse(input: &DataArray<f64>) -> DataArray<f64> {
    assert_eq!(input.num_components(), 9);
    let n = input.num_tuples();
    let mut data = Vec::with_capacity(n * 9);

    for i in 0..n {
        let m = input.tuple(i);
        let det = m[0] * (m[4] * m[8] - m[5] * m[7]) - m[1] * (m[3] * m[8] - m[5] * m[6])
            + m[2] * (m[3] * m[7] - m[4] * m[6]);

        if det.abs() < 1e-30 {
            data.extend_from_slice(&[0.0; 9]);
        } else {
            let inv_det = 1.0 / det;
            data.push((m[4] * m[8] - m[5] * m[7]) * inv_det);
            data.push((m[2] * m[7] - m[1] * m[8]) * inv_det);
            data.push((m[1] * m[5] - m[2] * m[4]) * inv_det);
            data.push((m[5] * m[6] - m[3] * m[8]) * inv_det);
            data.push((m[0] * m[8] - m[2] * m[6]) * inv_det);
            data.push((m[2] * m[3] - m[0] * m[5]) * inv_det);
            data.push((m[3] * m[7] - m[4] * m[6]) * inv_det);
            data.push((m[1] * m[6] - m[0] * m[7]) * inv_det);
            data.push((m[0] * m[4] - m[1] * m[3]) * inv_det);
        }
    }

    DataArray::from_vec("Inverse", data, 9)
}

/// Compute eigenvalues of 3x3 symmetric matrices (6-component: xx,yy,zz,xy,yz,xz).
/// Returns 3-component tuples (λ1 ≥ λ2 ≥ λ3).
pub fn symmetric_eigenvalues(input: &DataArray<f64>) -> DataArray<f64> {
    assert_eq!(input.num_components(), 6);
    let n = input.num_tuples();
    let mut data = Vec::with_capacity(n * 3);

    for i in 0..n {
        let t = input.tuple(i);
        let eigs = eigenvalues_3x3_symmetric(t[0], t[1], t[2], t[3], t[4], t[5]);
        data.extend_from_slice(&eigs);
    }

    DataArray::from_vec("Eigenvalues", data, 3)
}

/// Compute trace of 3x3 matrices (9-component).
pub fn matrix_trace(input: &DataArray<f64>) -> DataArray<f64> {
    assert_eq!(input.num_components(), 9);
    let n = input.num_tuples();
    let mut data = Vec::with_capacity(n);

    for i in 0..n {
        let m = input.tuple(i);
        data.push(m[0] + m[4] + m[8]);
    }

    DataArray::from_vec("Trace", data, 1)
}

/// Compute Frobenius norm of 3x3 matrices (9-component).
pub fn matrix_frobenius_norm(input: &DataArray<f64>) -> DataArray<f64> {
    assert_eq!(input.num_components(), 9);
    let n = input.num_tuples();
    let mut data = Vec::with_capacity(n);

    for i in 0..n {
        let m = input.tuple(i);
        let sum: f64 = m.iter().map(|v| v * v).sum();
        data.push(sum.sqrt());
    }

    DataArray::from_vec("FrobeniusNorm", data, 1)
}

fn eigenvalues_3x3_symmetric(
    a11: f64,
    a22: f64,
    a33: f64,
    a12: f64,
    a23: f64,
    a13: f64,
) -> [f64; 3] {
    let p1 = a12 * a12 + a13 * a13 + a23 * a23;
    if p1.abs() < 1e-30 {
        let mut vals = [a11, a22, a33];
        vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        return vals;
    }

    let q = (a11 + a22 + a33) / 3.0;
    let p2 = (a11 - q).powi(2) + (a22 - q).powi(2) + (a33 - q).powi(2) + 2.0 * p1;
    let p = (p2 / 6.0).sqrt();

    let b11 = (a11 - q) / p;
    let b22 = (a22 - q) / p;
    let b33 = (a33 - q) / p;
    let b12 = a12 / p;
    let b13 = a13 / p;
    let b23 = a23 / p;

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
    let eig2 = 3.0 * q - eig1 - eig3;

    let mut vals = [eig1, eig2, eig3];
    vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    vals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_determinant() {
        let m = DataArray::from_vec("m", vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0], 9);
        let det = matrix_determinant(&m);
        assert!((det.tuple(0)[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn identity_inverse() {
        let m = DataArray::from_vec("m", vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0], 9);
        let inv = matrix_inverse(&m);
        let t = inv.tuple(0);
        assert!((t[0] - 1.0).abs() < 1e-10);
        assert!((t[4] - 1.0).abs() < 1e-10);
        assert!((t[8] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn diagonal_eigenvalues() {
        let m = DataArray::from_vec("s", vec![3.0, 2.0, 1.0, 0.0, 0.0, 0.0], 6);
        let eigs = symmetric_eigenvalues(&m);
        let v = eigs.tuple(0);
        assert!((v[0] - 3.0).abs() < 1e-10);
        assert!((v[1] - 2.0).abs() < 1e-10);
        assert!((v[2] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn trace_identity() {
        let m = DataArray::from_vec("m", vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0], 9);
        let tr = matrix_trace(&m);
        assert!((tr.tuple(0)[0] - 3.0).abs() < 1e-10);
    }
}
