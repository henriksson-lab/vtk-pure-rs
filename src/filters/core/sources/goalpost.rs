//! Various goalpost/frame shapes.
use crate::data::{CellArray, Points, PolyData};
pub fn h_frame(width: f64, height: f64, crossbar_height: f64, _bar_radius: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let hw = width / 2.0;
    // Left upright
    let lb = pts.len();
    pts.push([-hw, 0.0, 0.0]);
    pts.push([-hw, 0.0, height]);
    lines.push_cell(&[lb as i64, (lb + 1) as i64]);
    // Right upright
    let rb = pts.len();
    pts.push([hw, 0.0, 0.0]);
    pts.push([hw, 0.0, height]);
    lines.push_cell(&[rb as i64, (rb + 1) as i64]);
    // Crossbar
    let cb = pts.len();
    pts.push([-hw, 0.0, crossbar_height]);
    pts.push([hw, 0.0, crossbar_height]);
    lines.push_cell(&[cb as i64, (cb + 1) as i64]);
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
pub fn rugby_posts(width: f64, height: f64, crossbar_height: f64) -> PolyData {
    h_frame(width, height, crossbar_height, 0.1)
}
pub fn portal_frame(width: f64, height: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let hw = width / 2.0;
    pts.push([-hw, 0.0, 0.0]);
    pts.push([-hw, 0.0, height]);
    pts.push([hw, 0.0, height]);
    pts.push([hw, 0.0, 0.0]);
    lines.push_cell(&[0, 1]);
    lines.push_cell(&[1, 2]);
    lines.push_cell(&[2, 3]);
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_h() {
        let h = h_frame(5.6, 10.0, 3.0, 0.1);
        assert_eq!(h.lines.num_cells(), 3);
    }
    #[test]
    fn test_rugby() {
        let r = rugby_posts(5.6, 15.0, 3.0);
        assert_eq!(r.lines.num_cells(), 3);
    }
    #[test]
    fn test_portal() {
        let p = portal_frame(4.0, 3.0);
        assert_eq!(p.lines.num_cells(), 3);
    }
}
