//! Rack (shelf/bookshelf) geometry source.
use crate::data::{CellArray, Points, PolyData};
pub fn rack(width: f64, height: f64, depth: f64, shelves: usize, thickness: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let shelf_gap = height / (shelves + 1) as f64;
    for i in 0..=shelves {
        let y = i as f64 * shelf_gap;
        let b = pts.len();
        pts.push([0.0, y, 0.0]);
        pts.push([width, y, 0.0]);
        pts.push([width, y, depth]);
        pts.push([0.0, y, depth]);
        pts.push([0.0, y + thickness, 0.0]);
        pts.push([width, y + thickness, 0.0]);
        pts.push([width, y + thickness, depth]);
        pts.push([0.0, y + thickness, depth]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
        polys.push_cell(&[f(0), f(4), f(7), f(3)]);
        polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    }
    // Side panels
    for side_x in [0.0, width - thickness] {
        let b = pts.len();
        pts.push([side_x, 0.0, 0.0]);
        pts.push([side_x + thickness, 0.0, 0.0]);
        pts.push([side_x + thickness, height, 0.0]);
        pts.push([side_x, height, 0.0]);
        pts.push([side_x, 0.0, depth]);
        pts.push([side_x + thickness, 0.0, depth]);
        pts.push([side_x + thickness, height, depth]);
        pts.push([side_x, height, depth]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(1), f(2), f(3)]);
        polys.push_cell(&[f(4), f(7), f(6), f(5)]);
        polys.push_cell(&[f(0), f(4), f(5), f(1)]);
        polys.push_cell(&[f(2), f(6), f(7), f(3)]);
        polys.push_cell(&[f(0), f(3), f(7), f(4)]);
        polys.push_cell(&[f(1), f(5), f(6), f(2)]);
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
    fn test_rack() {
        let r = rack(2.0, 3.0, 0.5, 3, 0.05);
        assert!(r.points.len() > 20);
        assert_eq!(r.polys.num_cells(), 36);
    }
}
