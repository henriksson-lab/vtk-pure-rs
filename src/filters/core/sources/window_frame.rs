//! Window frame (rectangular frame with optional crossbars).
use crate::data::{CellArray, Points, PolyData};
pub fn window_frame(width: f64, height: f64, frame_w: f64, depth: f64, crossbar: bool) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let fw = frame_w;
    let hd = depth / 2.0;
    // Helper: add a box
    let mut add_box = |x0: f64, y0: f64, x1: f64, y1: f64| {
        let b = pts.len();
        pts.push([x0, y0, -hd]);
        pts.push([x1, y0, -hd]);
        pts.push([x1, y1, -hd]);
        pts.push([x0, y1, -hd]);
        pts.push([x0, y0, hd]);
        pts.push([x1, y0, hd]);
        pts.push([x1, y1, hd]);
        pts.push([x0, y1, hd]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
        polys.push_cell(&[f(0), f(4), f(7), f(3)]);
        polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    };
    add_box(0.0, 0.0, width, fw); // bottom
    add_box(0.0, height - fw, width, height); // top
    add_box(0.0, fw, fw, height - fw); // left
    add_box(width - fw, fw, width, height - fw); // right
    if crossbar {
        add_box(fw, (height - fw) / 2.0, width - fw, (height + fw) / 2.0); // horizontal bar
        add_box((width - fw) / 2.0, fw, (width + fw) / 2.0, height - fw); // vertical bar
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
    fn test_plain() {
        let f = window_frame(2.0, 3.0, 0.1, 0.05, false);
        assert_eq!(f.polys.num_cells(), 24);
    }
    #[test]
    fn test_cross() {
        let f = window_frame(2.0, 3.0, 0.1, 0.05, true);
        assert_eq!(f.polys.num_cells(), 36);
    }
}
