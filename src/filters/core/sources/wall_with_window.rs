//! Wall with rectangular window opening.
use crate::data::{CellArray, Points, PolyData};
pub fn wall_with_window(
    w: f64,
    h: f64,
    win_x: f64,
    win_y: f64,
    win_w: f64,
    win_h: f64,
) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Outer corners
    pts.push([0.0, 0.0, 0.0]);
    pts.push([w, 0.0, 0.0]);
    pts.push([w, h, 0.0]);
    pts.push([0.0, h, 0.0]);
    // Window corners
    pts.push([win_x, win_y, 0.0]);
    pts.push([win_x + win_w, win_y, 0.0]);
    pts.push([win_x + win_w, win_y + win_h, 0.0]);
    pts.push([win_x, win_y + win_h, 0.0]);
    // Bottom strip: 0-1-5-4
    polys.push_cell(&[0, 1, 5, 4]);
    // Right strip: 1-2-6-5
    polys.push_cell(&[1, 2, 6, 5]);
    // Top strip: 2-3-7-6
    polys.push_cell(&[2, 3, 7, 6]);
    // Left strip: 3-0-4-7
    polys.push_cell(&[3, 0, 4, 7]);
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
        let w = wall_with_window(5.0, 3.0, 1.0, 1.0, 2.0, 1.5);
        assert_eq!(w.points.len(), 8);
        assert_eq!(w.polys.num_cells(), 4);
    }
}
