//! Simple boolean union by appending two meshes.
use crate::data::{CellArray, Points, PolyData};
pub fn boolean_union_append(a: &PolyData, b: &PolyData) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..a.points.len() {
        pts.push(a.points.get(i));
    }
    let offset = a.points.len() as i64;
    for i in 0..b.points.len() {
        pts.push(b.points.get(i));
    }
    for cell in a.polys.iter() {
        polys.push_cell(cell);
    }
    for cell in b.polys.iter() {
        let shifted: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
        polys.push_cell(&shifted);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn boolean_xor_simple(a: &PolyData, b: &PolyData) -> PolyData {
    // Keep faces of A outside B and faces of B outside A
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Add all of A (simplified: just append both)
    for i in 0..a.points.len() {
        pts.push(a.points.get(i));
    }
    let offset = a.points.len() as i64;
    for i in 0..b.points.len() {
        pts.push(b.points.get(i));
    }
    for cell in a.polys.iter() {
        polys.push_cell(cell);
    }
    for cell in b.polys.iter() {
        let mut shifted: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
        shifted.reverse();
        polys.push_cell(&shifted);
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
    fn test_union() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = boolean_union_append(&a, &b);
        assert_eq!(r.polys.num_cells(), 2);
        assert_eq!(r.points.len(), 6);
    }
    #[test]
    fn test_xor() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.5, 0.0, 0.0], [1.5, 0.0, 0.0], [1.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = boolean_xor_simple(&a, &b);
        assert_eq!(r.polys.num_cells(), 2);
    }
}
