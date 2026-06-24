//! Simple mesh transform utilities: translate, uniform scale, rotate around axis.
use crate::data::{Points, PolyData};

pub fn translate(mesh: &PolyData, dx: f64, dy: f64, dz: f64) -> PolyData {
    let n = mesh.points.len();
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([p[0] + dx, p[1] + dy, p[2] + dz]);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
}

pub fn uniform_scale(mesh: &PolyData, factor: f64) -> PolyData {
    let n = mesh.points.len();
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([p[0] * factor, p[1] * factor, p[2] * factor]);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
}

pub fn rotate_z(mesh: &PolyData, angle_rad: f64) -> PolyData {
    let n = mesh.points.len();
    let c = angle_rad.cos();
    let s = angle_rad.sin();
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([p[0] * c - p[1] * s, p[0] * s + p[1] * c, p[2]]);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
}

pub fn rotate_y(mesh: &PolyData, angle_rad: f64) -> PolyData {
    let n = mesh.points.len();
    let c = angle_rad.cos();
    let s = angle_rad.sin();
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([p[0] * c + p[2] * s, p[1], -p[0] * s + p[2] * c]);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
}

pub fn rotate_x(mesh: &PolyData, angle_rad: f64) -> PolyData {
    let n = mesh.points.len();
    let c = angle_rad.cos();
    let s = angle_rad.sin();
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([p[0], p[1] * c - p[2] * s, p[1] * s + p[2] * c]);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
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
        let r = translate(&mesh, 10.0, 0.0, 0.0);
        assert!((r.points.get(0)[0] - 10.0).abs() < 1e-9);
    }
    #[test]
    fn test_scale() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = uniform_scale(&mesh, 3.0);
        assert!((r.points.get(0)[0] - 3.0).abs() < 1e-9);
    }
    #[test]
    fn test_rotate() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = rotate_z(&mesh, std::f64::consts::PI / 2.0);
        assert!((r.points.get(0)[0]).abs() < 1e-9); // x=1 rotated 90deg -> x~0
        assert!((r.points.get(0)[1] - 1.0).abs() < 1e-9); // y~1
    }
}
