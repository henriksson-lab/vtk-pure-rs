//! Make mesh symmetric by mirroring and merging.
use crate::data::{CellArray, Points, PolyData};
pub fn symmetrize_x(mesh: &PolyData) -> PolyData {
    symmetrize(mesh, 0)
}
pub fn symmetrize_y(mesh: &PolyData) -> PolyData {
    symmetrize(mesh, 1)
}
pub fn symmetrize_z(mesh: &PolyData) -> PolyData {
    symmetrize(mesh, 2)
}
fn symmetrize(mesh: &PolyData, axis: usize) -> PolyData {
    let n = mesh.points.len();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..n {
        pts.push(mesh.points.get(i));
    }
    for i in 0..n {
        let mut p = mesh.points.get(i);
        p[axis] = -p[axis];
        pts.push(p);
    }
    for cell in mesh.polys.iter() {
        polys.push_cell(cell);
    }
    for cell in mesh.polys.iter() {
        let mut rev: Vec<i64> = cell.iter().map(|&v| v + n as i64).collect();
        rev.reverse();
        polys.push_cell(&rev);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = symmetrize_x(&m);
        assert_eq!(r.points.len(), 6);
        assert_eq!(r.polys.num_cells(), 2);
    }
}
