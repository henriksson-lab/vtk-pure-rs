//! Stockless ship anchor (modern type).
use crate::data::{CellArray, Points, PolyData};

pub fn ship_anchor(height: f64, fluke_span: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let hw = fluke_span / 2.0;
    // Shank (vertical bar)
    let s0 = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let s1 = pts.len();
    pts.push([0.0, 0.0, height]);
    lines.push_cell(&[s0 as i64, s1 as i64]);
    // Shackle at top (ring)
    let ring_r = height * 0.08;
    let na = 8;
    let rb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([ring_r * a.cos(), 0.0, height + ring_r + ring_r * a.sin()]);
    }
    for j in 0..na {
        lines.push_cell(&[(rb + j) as i64, (rb + (j + 1) % na) as i64]);
    }
    // Crown (connection point at bottom)
    let crown = pts.len();
    pts.push([0.0, 0.0, -height * 0.05]);
    lines.push_cell(&[s0 as i64, crown as i64]);
    // Flukes (two triangular blades)
    for &sx in &[-1.0f64, 1.0] {
        let fb = pts.len();
        pts.push([0.0, 0.0, -height * 0.05]); // base at crown
        pts.push([sx * hw, 0.0, -height * 0.1]); // tip
        pts.push([sx * hw * 0.5, 0.0, height * 0.15]); // upper edge
        polys.push_cell(&[fb as i64, (fb + 1) as i64, (fb + 2) as i64]);
    }
    // Tripping palm (small triangles on flukes)
    for &sx in &[-1.0f64, 1.0] {
        let tp = pts.len();
        pts.push([sx * hw * 0.4, hw * 0.1, 0.0]);
        pts.push([sx * hw * 0.6, hw * 0.1, -height * 0.05]);
        pts.push([sx * hw * 0.5, -hw * 0.1, -height * 0.02]);
        polys.push_cell(&[tp as i64, (tp + 1) as i64, (tp + 2) as i64]);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ship_anchor() {
        let m = ship_anchor(5.0, 3.0);
        assert!(m.points.len() > 15);
        assert!(m.polys.num_cells() >= 4);
        assert!(m.lines.num_cells() > 5);
    }
}
