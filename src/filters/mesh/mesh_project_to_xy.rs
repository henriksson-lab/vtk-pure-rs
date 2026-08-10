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
pub use crate::filters::mesh::project_to_plane::project_to_plane;
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
}
