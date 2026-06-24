//! Ramp/inclined plane geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn ramp(width: f64, length: f64, height: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let hw = width / 2.0;
    pts.push([-hw, 0.0, 0.0]);
    pts.push([hw, 0.0, 0.0]);
    pts.push([hw, length, height]);
    pts.push([-hw, length, height]);
    pts.push([-hw, length, 0.0]);
    pts.push([hw, length, 0.0]);
    polys.push_cell(&[0, 1, 2, 3]); // top slope
    polys.push_cell(&[0, 4, 5, 1]); // bottom
    polys.push_cell(&[0, 3, 4]); // left triangle
    polys.push_cell(&[1, 5, 2]); // right triangle
    polys.push_cell(&[3, 2, 5, 4]); // back wall
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn wedge_ramp(width: f64, length: f64, height: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let hw = width / 2.0;
    pts.push([-hw, 0.0, 0.0]);
    pts.push([hw, 0.0, 0.0]);
    pts.push([hw, length, 0.0]);
    pts.push([-hw, length, 0.0]);
    pts.push([-hw, 0.0, height]);
    pts.push([hw, 0.0, height]);
    polys.push_cell(&[0, 1, 2, 3]); // bottom
    polys.push_cell(&[0, 4, 5, 1]); // front
    polys.push_cell(&[0, 3, 4]); // left
    polys.push_cell(&[1, 5, 2]); // right
    polys.push_cell(&[4, 3, 2, 5]); // slope
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ramp() {
        let r = ramp(2.0, 5.0, 1.0);
        assert_eq!(r.points.len(), 6);
        assert_eq!(r.polys.num_cells(), 5);
    }
    #[test]
    fn test_wedge() {
        let w = wedge_ramp(2.0, 5.0, 1.0);
        assert_eq!(w.points.len(), 6);
        assert_eq!(w.polys.num_cells(), 5);
    }
}
