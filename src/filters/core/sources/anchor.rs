//! Ship anchor geometry (simplified).
use crate::data::{CellArray, Points, PolyData};
pub fn anchor(height: f64, arm_span: f64, shank_width: f64) -> PolyData {
    let h = height;
    let hw = arm_span / 2.0;
    let sw = shank_width / 2.0;
    let _t = shank_width * 0.5;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // Shank (vertical bar)
    let b = pts.len();
    pts.push([-sw, 0.0, 0.0]);
    pts.push([sw, 0.0, 0.0]);
    pts.push([sw, 0.0, h]);
    pts.push([-sw, 0.0, h]);
    polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64, (b + 3) as i64]);
    // Ring at top
    let ring_r = shank_width;
    let ring_res = 12;
    for i in 0..ring_res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / ring_res as f64;
        let j = (i + 1) % ring_res;
        let a2 = 2.0 * std::f64::consts::PI * j as f64 / ring_res as f64;
        let rb = pts.len();
        pts.push([ring_r * a.cos(), 0.0, h + ring_r + ring_r * a.sin()]);
        pts.push([ring_r * a2.cos(), 0.0, h + ring_r + ring_r * a2.sin()]);
        lines.push_cell(&[rb as i64, (rb + 1) as i64]);
    }
    // Crown (cross bar at base)
    let cb = pts.len();
    pts.push([-hw, 0.0, h * 0.15]);
    pts.push([hw, 0.0, h * 0.15]);
    lines.push_cell(&[cb as i64, (cb + 1) as i64]);
    // Arms (curved down)
    for side in [-1.0f64, 1.0] {
        let ab = pts.len();
        pts.push([side * hw, 0.0, h * 0.15]);
        pts.push([side * hw * 0.8, 0.0, 0.0]);
        pts.push([side * hw * 0.5, 0.0, -h * 0.1]);
        lines.push_cell(&[ab as i64, (ab + 1) as i64]);
        lines.push_cell(&[(ab + 1) as i64, (ab + 2) as i64]);
    }
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
        let a = anchor(3.0, 2.0, 0.3);
        assert!(a.lines.num_cells() > 10);
    }
}
