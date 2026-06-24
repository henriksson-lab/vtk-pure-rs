//! Twist mesh around an axis.
use crate::data::PolyData;
pub fn twist(mesh: &PolyData, axis: usize, angle_per_unit: f64) -> PolyData {
    let n = mesh.points.len();
    let mut r = mesh.clone();
    for i in 0..n {
        let p = r.points.get(i);
        let t = p[axis] * angle_per_unit;
        let c = t.cos();
        let s = t.sin();
        let np = match axis {
            0 => [p[0], c * p[1] - s * p[2], s * p[1] + c * p[2]],
            1 => [c * p[0] + s * p[2], p[1], -s * p[0] + c * p[2]],
            _ => [c * p[0] - s * p[1], s * p[0] + c * p[1], p[2]],
        };
        r.points.set(i, np);
    }
    r
}
pub fn taper(mesh: &PolyData, axis: usize, scale_per_unit: f64) -> PolyData {
    let n = mesh.points.len();
    let mut r = mesh.clone();
    for i in 0..n {
        let p = r.points.get(i);
        let s = 1.0 + p[axis] * scale_per_unit;
        let s = s.max(0.01);
        let np = match axis {
            0 => [p[0], p[1] * s, p[2] * s],
            1 => [p[0] * s, p[1], p[2] * s],
            _ => [p[0] * s, p[1] * s, p[2]],
        };
        r.points.set(i, np);
    }
    r
}
pub fn bend_mesh(mesh: &PolyData, axis: usize, bend_axis: usize, curvature: f64) -> PolyData {
    let n = mesh.points.len();
    let mut r = mesh.clone();
    for i in 0..n {
        let p = r.points.get(i);
        let t = p[axis] * curvature;
        let radius = if curvature.abs() > 1e-15 {
            1.0 / curvature
        } else {
            1e15
        };
        let mut np = p;
        np[bend_axis] += radius * (1.0 - t.cos());
        np[axis] = radius * t.sin();
        r.points.set(i, np);
    }
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_twist() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let r = twist(&m, 2, 1.0);
        assert_eq!(r.points.len(), 3);
    }
    #[test]
    fn test_taper() {
        let m = PolyData::from_triangles(
            vec![[1.0, 1.0, 0.0], [1.0, 1.0, 1.0], [1.0, 0.0, 0.5]],
            vec![[0, 1, 2]],
        );
        let r = taper(&m, 2, 0.5);
        assert_eq!(r.points.len(), 3);
    }
    #[test]
    fn test_bend() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let r = bend_mesh(&m, 2, 1, 0.5);
        assert_eq!(r.points.len(), 3);
    }
}
