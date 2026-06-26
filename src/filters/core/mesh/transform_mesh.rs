//! Mesh transformation utilities (translate, rotate, scale, transform matrix).

use crate::data::PolyData;

/// Translate all points by an offset.
pub fn translate_mesh(mesh: &PolyData, offset: [f64; 3]) -> PolyData {
    let mut result = mesh.clone();
    for i in 0..result.points.len() {
        let p = result.points.get(i);
        result
            .points
            .set(i, [p[0] + offset[0], p[1] + offset[1], p[2] + offset[2]]);
    }
    result
}

/// Scale all points relative to origin.
pub fn scale_mesh(mesh: &PolyData, scale: [f64; 3]) -> PolyData {
    let mut result = mesh.clone();
    for i in 0..result.points.len() {
        let p = result.points.get(i);
        result
            .points
            .set(i, [p[0] * scale[0], p[1] * scale[1], p[2] * scale[2]]);
    }
    result
}

/// Scale uniformly.
pub fn scale_uniform(mesh: &PolyData, factor: f64) -> PolyData {
    scale_mesh(mesh, [factor, factor, factor])
}

/// Rotate around X axis by angle (radians).
pub fn rotate_x(mesh: &PolyData, angle: f64) -> PolyData {
    let c = angle.cos();
    let s = angle.sin();
    let mut result = mesh.clone();
    for i in 0..result.points.len() {
        let p = result.points.get(i);
        result
            .points
            .set(i, [p[0], c * p[1] - s * p[2], s * p[1] + c * p[2]]);
    }
    result
}

/// Rotate around Y axis by angle (radians).
pub fn rotate_y(mesh: &PolyData, angle: f64) -> PolyData {
    let c = angle.cos();
    let s = angle.sin();
    let mut result = mesh.clone();
    for i in 0..result.points.len() {
        let p = result.points.get(i);
        result
            .points
            .set(i, [c * p[0] + s * p[2], p[1], -s * p[0] + c * p[2]]);
    }
    result
}

/// Rotate around Z axis by angle (radians).
pub fn rotate_z(mesh: &PolyData, angle: f64) -> PolyData {
    let c = angle.cos();
    let s = angle.sin();
    let mut result = mesh.clone();
    for i in 0..result.points.len() {
        let p = result.points.get(i);
        result
            .points
            .set(i, [c * p[0] - s * p[1], s * p[0] + c * p[1], p[2]]);
    }
    result
}

/// Apply a 4x4 transformation matrix in row-major order.
pub fn transform_by_matrix(mesh: &PolyData, m: &[[f64; 4]; 4]) -> PolyData {
    let mut result = mesh.clone();
    for i in 0..result.points.len() {
        let p = result.points.get(i);
        let x = m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3];
        let y = m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3];
        let z = m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3];
        let w = m[3][0] * p[0] + m[3][1] * p[1] + m[3][2] * p[2] + m[3][3];
        if w.abs() > 1e-15 {
            let inv_w = 1.0 / w;
            result.points.set(i, [x * inv_w, y * inv_w, z * inv_w]);
        } else {
            result.points.set(i, [x, y, z]);
        }
    }
    result
}

/// Center mesh at origin.
pub fn center_at_origin(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut c = [0.0, 0.0, 0.0];
    for i in 0..n {
        let p = mesh.points.get(i);
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    let nf = n as f64;
    translate_mesh(mesh, [-c[0] / nf, -c[1] / nf, -c[2] / nf])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_translate() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = translate_mesh(&mesh, [10.0, 20.0, 30.0]);
        let p = r.points.get(0);
        assert!((p[0] - 10.0).abs() < 1e-10);
    }
    #[test]
    fn test_scale() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );
        let r = scale_uniform(&mesh, 2.0);
        let p = r.points.get(0);
        assert!((p[0] - 2.0).abs() < 1e-10);
    }
    #[test]
    fn test_rotate_z() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = rotate_z(&mesh, std::f64::consts::FRAC_PI_2);
        let p = r.points.get(0);
        assert!(p[0].abs() < 1e-10); // (1,0) -> (0,1)
        assert!((p[1] - 1.0).abs() < 1e-10);
    }
    #[test]
    fn test_homogeneous_matrix_divides_by_w() {
        let mesh = PolyData::from_triangles(
            vec![[2.0, 4.0, 6.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let matrix = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 2.0],
        ];
        let r = transform_by_matrix(&mesh, &matrix);
        let p = r.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-10);
        assert!((p[1] - 2.0).abs() < 1e-10);
        assert!((p[2] - 3.0).abs() < 1e-10);
    }
    #[test]
    fn test_homogeneous_matrix_handles_zero_w() {
        let mesh = PolyData::from_triangles(
            vec![[2.0, 4.0, 6.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let matrix = [
            [1.0, 0.0, 0.0, 10.0],
            [0.0, 1.0, 0.0, 20.0],
            [0.0, 0.0, 1.0, 30.0],
            [0.0, 0.0, 0.0, 0.0],
        ];
        let r = transform_by_matrix(&mesh, &matrix);
        let p = r.points.get(0);
        assert_eq!(p, [12.0, 24.0, 36.0]);
    }
    #[test]
    fn test_center() {
        let mesh = PolyData::from_triangles(
            vec![[2.0, 2.0, 2.0], [4.0, 2.0, 2.0], [3.0, 4.0, 2.0]],
            vec![[0, 1, 2]],
        );
        let r = center_at_origin(&mesh);
        let mut cx = 0.0;
        for i in 0..3 {
            cx += r.points.get(i)[0];
        }
        assert!(cx.abs() < 1e-10);
    }
}
