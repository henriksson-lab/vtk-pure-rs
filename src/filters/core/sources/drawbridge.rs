//! Drawbridge geometry (bridge deck + towers + chains).
use crate::data::{CellArray, Points, PolyData};
pub fn drawbridge(span: f64, width: f64, tower_h: f64, deck_thickness: f64) -> PolyData {
    let hs = span / 2.0;
    let hw = width / 2.0;
    let dt = deck_thickness;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Deck (two halves that can raise)
    for &side in &[-1.0f64, 1.0] {
        let x0 = 0.0f64;
        let x1 = side * hs;
        let b = pts.len();
        pts.push([x0.min(x1), -hw, 0.0]);
        pts.push([x0.max(x1), -hw, 0.0]);
        pts.push([x0.max(x1), hw, 0.0]);
        pts.push([x0.min(x1), hw, 0.0]);
        pts.push([x0.min(x1), -hw, dt]);
        pts.push([x0.max(x1), -hw, dt]);
        pts.push([x0.max(x1), hw, dt]);
        pts.push([x0.min(x1), hw, dt]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    }
    // Towers
    let tw = width * 0.15;
    for &x in &[-hs - tw, hs] {
        let b = pts.len();
        pts.push([x, -hw, 0.0]);
        pts.push([x + tw, -hw, 0.0]);
        pts.push([x + tw, hw, 0.0]);
        pts.push([x, hw, 0.0]);
        pts.push([x, -hw, tower_h]);
        pts.push([x + tw, -hw, tower_h]);
        pts.push([x + tw, hw, tower_h]);
        pts.push([x, hw, tower_h]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(1), f(2), f(6), f(5)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
        polys.push_cell(&[f(3), f(0), f(4), f(7)]);
    }
    // Chains
    for &(tx, dx) in &[(-hs - tw / 2.0, 0.0), (hs + tw / 2.0, 0.0)] {
        let cb = pts.len();
        pts.push([tx, 0.0, tower_h]);
        pts.push([dx, 0.0, dt]);
        lines.push_cell(&[cb as i64, (cb + 1) as i64]);
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
        let d = drawbridge(10.0, 4.0, 8.0, 0.3);
        assert!(d.polys.num_cells() > 10);
        assert!(d.lines.num_cells() >= 2);
    }
}
