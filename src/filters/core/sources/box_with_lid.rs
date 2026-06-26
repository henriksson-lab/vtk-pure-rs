//! Open box with optional lid geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn open_box(width: f64, depth: f64, height: f64, wall_thickness: f64) -> PolyData {
    let w = width.max(0.0);
    let d = depth.max(0.0);
    let h = height.max(0.0);
    let t = wall_thickness.max(0.0).min(w * 0.5).min(d * 0.5).min(h);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut add_quad = |a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]| {
        let i = pts.len();
        pts.push(a);
        pts.push(b);
        pts.push(c);
        pts.push(d);
        polys.push_cell(&[i as i64, (i + 1) as i64, (i + 2) as i64, (i + 3) as i64]);
    };
    // Outer walls
    add_quad([0.0, 0.0, 0.0], [w, 0.0, 0.0], [w, 0.0, h], [0.0, 0.0, h]); // front
    add_quad([w, 0.0, 0.0], [w, d, 0.0], [w, d, h], [w, 0.0, h]); // right
    add_quad([w, d, 0.0], [0.0, d, 0.0], [0.0, d, h], [w, d, h]); // back
    add_quad([0.0, d, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, h], [0.0, d, h]); // left
                                                                          // Bottom
    add_quad([0.0, 0.0, 0.0], [0.0, d, 0.0], [w, d, 0.0], [w, 0.0, 0.0]);
    // Inner walls
    add_quad([t, t, t], [t, t, h], [w - t, t, h], [w - t, t, t]);
    add_quad(
        [w - t, t, t],
        [w - t, t, h],
        [w - t, d - t, h],
        [w - t, d - t, t],
    );
    add_quad(
        [w - t, d - t, t],
        [w - t, d - t, h],
        [t, d - t, h],
        [t, d - t, t],
    );
    add_quad([t, d - t, t], [t, d - t, h], [t, t, h], [t, t, t]);
    // Inner bottom
    add_quad([t, t, t], [w - t, t, t], [w - t, d - t, t], [t, d - t, t]);
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
        let b = open_box(2.0, 1.5, 1.0, 0.1);
        assert!(b.polys.num_cells() >= 10);
    }
}
