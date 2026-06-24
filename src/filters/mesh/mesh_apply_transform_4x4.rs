//! Apply 4x4 transformation matrices.
use crate::data::PolyData;
pub fn apply_transform(mesh: &PolyData, m: &[[f64; 4]; 4]) -> PolyData {
    let n = mesh.points.len();
    let mut r = mesh.clone();
    for i in 0..n {
        let p = mesh.points.get(i);
        let x = m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3];
        let y = m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3];
        let z = m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3];
        r.points.set(i, [x, y, z]);
    }
    r
}
pub fn rotation_matrix_z(angle: f64) -> [[f64; 4]; 4] {
    let c = angle.cos();
    let s = angle.sin();
    [
        [c, -s, 0.0, 0.0],
        [s, c, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
pub fn rotation_matrix_y(angle: f64) -> [[f64; 4]; 4] {
    let c = angle.cos();
    let s = angle.sin();
    [
        [c, 0.0, s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-s, 0.0, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
pub fn rotation_matrix_x(angle: f64) -> [[f64; 4]; 4] {
    let c = angle.cos();
    let s = angle.sin();
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, c, -s, 0.0],
        [0.0, s, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
pub fn translation_matrix(tx: f64, ty: f64, tz: f64) -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, tx],
        [0.0, 1.0, 0.0, ty],
        [0.0, 0.0, 1.0, tz],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
pub fn scale_matrix(sx: f64, sy: f64, sz: f64) -> [[f64; 4]; 4] {
    [
        [sx, 0.0, 0.0, 0.0],
        [0.0, sy, 0.0, 0.0],
        [0.0, 0.0, sz, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
pub fn multiply_matrices(a: &[[f64; 4]; 4], b: &[[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut r = [[0.0f64; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                r[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_translate() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let t = translation_matrix(10.0, 20.0, 30.0);
        let r = apply_transform(&m, &t);
        let p = r.points.get(0);
        assert!((p[0] - 10.0).abs() < 1e-10);
    }
    #[test]
    fn test_rotate() {
        let m = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let rz = rotation_matrix_z(std::f64::consts::FRAC_PI_2);
        let r = apply_transform(&m, &rz);
        let p = r.points.get(0);
        assert!(p[0].abs() < 1e-10);
        assert!((p[1] - 1.0).abs() < 1e-10);
    }
    #[test]
    fn test_compose() {
        let t = translation_matrix(5.0, 0.0, 0.0);
        let s = scale_matrix(2.0, 2.0, 2.0);
        let c = multiply_matrices(&t, &s);
        assert!((c[0][3] - 5.0).abs() < 1e-10);
    }
}
