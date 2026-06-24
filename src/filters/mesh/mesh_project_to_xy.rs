//! Project mesh vertices onto XY plane (flatten Z).
use crate::data::PolyData;
pub fn project_to_xy(mesh: &PolyData) -> PolyData {
    let mut r = mesh.clone();
    for i in 0..r.points.len() {
        let p = r.points.get(i);
        r.points.set(i, [p[0], p[1], 0.0]);
    }
    r
}
pub fn project_to_xz(mesh: &PolyData) -> PolyData {
    let mut r = mesh.clone();
    for i in 0..r.points.len() {
        let p = r.points.get(i);
        r.points.set(i, [p[0], 0.0, p[2]]);
    }
    r
}
pub fn project_to_yz(mesh: &PolyData) -> PolyData {
    let mut r = mesh.clone();
    for i in 0..r.points.len() {
        let p = r.points.get(i);
        r.points.set(i, [0.0, p[1], p[2]]);
    }
    r
}
pub fn project_to_plane(mesh: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    let nl = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2])
        .sqrt()
        .max(1e-15);
    let n = [normal[0] / nl, normal[1] / nl, normal[2] / nl];
    let mut r = mesh.clone();
    for i in 0..r.points.len() {
        let p = r.points.get(i);
        let d = (p[0] - origin[0]) * n[0] + (p[1] - origin[1]) * n[1] + (p[2] - origin[2]) * n[2];
        r.points
            .set(i, [p[0] - d * n[0], p[1] - d * n[1], p[2] - d * n[2]]);
    }
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_xy() {
        let m = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );
        let r = project_to_xy(&m);
        assert!((r.points.get(0)[2]).abs() < 1e-10);
    }
    #[test]
    fn test_plane() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 5.0], [1.0, 0.0, 5.0], [0.5, 1.0, 5.0]],
            vec![[0, 1, 2]],
        );
        let r = project_to_plane(&m, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert!((r.points.get(0)[2]).abs() < 1e-10);
    }
}
