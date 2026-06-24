//! Glass pyramid (Louvre-style) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn glass_pyramid(base: f64, height: f64, segments: usize) -> PolyData {
    let ns = segments.max(1);
    let hb = base / 2.0;
    let _step = base / ns as f64;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Frame edges (wireframe pyramid with horizontal rings)
    pts.push([0.0, 0.0, height]); // apex
                                  // Base corners
    pts.push([-hb, -hb, 0.0]);
    pts.push([hb, -hb, 0.0]);
    pts.push([hb, hb, 0.0]);
    pts.push([-hb, hb, 0.0]);
    // Edge lines to apex
    for i in 1..=4 {
        lines.push_cell(&[0, i as i64]);
    }
    // Base edges
    lines.push_cell(&[1, 2]);
    lines.push_cell(&[2, 3]);
    lines.push_cell(&[3, 4]);
    lines.push_cell(&[4, 1]);
    // Horizontal rings at each segment level
    for si in 1..ns {
        let t = si as f64 / ns as f64;
        let hw = hb * (1.0 - t);
        let z = t * height;
        let rb = pts.len();
        pts.push([-hw, -hw, z]);
        pts.push([hw, -hw, z]);
        pts.push([hw, hw, z]);
        pts.push([-hw, hw, z]);
        for i in 0..4 {
            lines.push_cell(&[(rb + i) as i64, (rb + (i + 1) % 4) as i64]);
        }
    }
    // Glass panels (transparent quads)
    polys.push_cell(&[0, 1, 2]);
    polys.push_cell(&[0, 2, 3]);
    polys.push_cell(&[0, 3, 4]);
    polys.push_cell(&[0, 4, 1]);
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let p = glass_pyramid(10.0, 7.0, 4);
        assert!(p.lines.num_cells() > 10);
        assert_eq!(p.polys.num_cells(), 4);
    }
}
