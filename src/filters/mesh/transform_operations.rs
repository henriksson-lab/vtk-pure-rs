//! Geometric transform operations: translate, rotate, scale, mirror.

use crate::data::{Points, PolyData};

/// Translate mesh by offset.
pub fn translate_mesh(mesh: &PolyData, offset: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([p[0] + offset[0], p[1] + offset[1], p[2] + offset[2]]);
    }
    let mut r = mesh.clone();
    r.points = pts;
    r
}

/// Scale mesh uniformly from center.
pub fn scale_mesh_uniform(mesh: &PolyData, factor: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut c = [0.0; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            c[j] += p[j];
        }
    }
    let nf = n as f64;
    for j in 0..3 {
        c[j] /= nf;
    }
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            c[0] + (p[0] - c[0]) * factor,
            c[1] + (p[1] - c[1]) * factor,
            c[2] + (p[2] - c[2]) * factor,
        ]);
    }
    let mut r = mesh.clone();
    r.points = pts;
    r
}

/// Scale mesh non-uniformly from center.
pub fn scale_mesh(mesh: &PolyData, factors: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut c = [0.0; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            c[j] += p[j];
        }
    }
    let nf = n as f64;
    for j in 0..3 {
        c[j] /= nf;
    }
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            c[0] + (p[0] - c[0]) * factors[0],
            c[1] + (p[1] - c[1]) * factors[1],
            c[2] + (p[2] - c[2]) * factors[2],
        ]);
    }
    let mut r = mesh.clone();
    r.points = pts;
    r
}

/// Rotate mesh around an axis by angle (radians) using Rodrigues' formula.
pub fn rotate_mesh(mesh: &PolyData, axis: [f64; 3], angle: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let al = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if al < 1e-15 {
        return mesh.clone();
    }
    let k = [axis[0] / al, axis[1] / al, axis[2] / al];
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    let mut c = [0.0; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            c[j] += p[j];
        }
    }
    let nf = n as f64;
    for j in 0..3 {
        c[j] /= nf;
    }

    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        let v = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
        let dot_kv = k[0] * v[0] + k[1] * v[1] + k[2] * v[2];
        let cross = [
            k[1] * v[2] - k[2] * v[1],
            k[2] * v[0] - k[0] * v[2],
            k[0] * v[1] - k[1] * v[0],
        ];
        let rv = [
            v[0] * cos_a + cross[0] * sin_a + k[0] * dot_kv * (1.0 - cos_a),
            v[1] * cos_a + cross[1] * sin_a + k[1] * dot_kv * (1.0 - cos_a),
            v[2] * cos_a + cross[2] * sin_a + k[2] * dot_kv * (1.0 - cos_a),
        ];
        pts.push([rv[0] + c[0], rv[1] + c[1], rv[2] + c[2]]);
    }
    let mut r = mesh.clone();
    r.points = pts;
    r
}

/// Mirror mesh across a coordinate plane.
pub fn mirror_mesh(mesh: &PolyData, plane: usize) -> PolyData {
    // 0=YZ, 1=XZ, 2=XY
    let n = mesh.points.len();
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let mut p = mesh.points.get(i);
        p[plane] = -p[plane];
        pts.push(p);
    }
    let mut r = mesh.clone();
    r.points = pts;
    // Reverse face winding
    let mut polys = crate::data::CellArray::new();
    for cell in r.polys.iter() {
        let rev: Vec<i64> = cell.iter().rev().cloned().collect();
        polys.push_cell(&rev);
    }
    r.polys = polys;
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn translate() {
        let mesh = PolyData::from_points(vec![[1.0, 2.0, 3.0]]);
        let result = translate_mesh(&mesh, [10.0, 20.0, 30.0]);
        let p = result.points.get(0);
        assert!((p[0] - 11.0).abs() < 0.01);
    }
    #[test]
    fn scale_uniform() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let result = scale_mesh_uniform(&mesh, 2.0);
        let p0 = result.points.get(0);
        let p1 = result.points.get(1);
        assert!((p1[0] - p0[0] - 4.0).abs() < 0.01);
    }
    #[test]
    fn rotate_90() {
        let mesh = PolyData::from_points(vec![[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]]);
        let result = rotate_mesh(&mesh, [0.0, 0.0, 1.0], std::f64::consts::FRAC_PI_2);
        let p = result.points.get(0);
        assert!(p[1].abs() > 0.9); // X rotated to Y
    }
    #[test]
    fn mirror_yz() {
        let mesh = PolyData::from_points(vec![[1.0, 2.0, 3.0]]);
        let result = mirror_mesh(&mesh, 0);
        assert!((result.points.get(0)[0] + 1.0).abs() < 0.01);
    }
}
