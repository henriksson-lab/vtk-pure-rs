//! Compass rose (direction indicator) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn compass_rose(radius: f64, num_points: usize) -> PolyData {
    let np = num_points.max(4);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    pts.push([0.0, 0.0, 0.0]); // center
    let inner_r = radius * 0.3;
    for i in 0..np * 2 {
        let a = std::f64::consts::PI * i as f64 / np as f64;
        let r = if i % 2 == 0 { radius } else { inner_r };
        pts.push([r * a.cos(), r * a.sin(), 0.0]);
    }
    for i in 0..np * 2 {
        let j = if i + 1 < np * 2 { i + 2 } else { 1 };
        polys.push_cell(&[0, (i + 1) as i64, j as i64]);
    }
    // Cardinal direction markers (taller points at N,E,S,W)
    for ci in 0..4 {
        let a = std::f64::consts::FRAC_PI_2 * ci as f64;
        let _idx = pts.len();
        pts.push([radius * 1.15 * a.cos(), radius * 1.15 * a.sin(), 0.01]);
        let _prev_idx = ci * np / 2 * 2 + 1; // approximate
                                             // Just mark with a point
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
    fn test_4() {
        let c = compass_rose(2.0, 4);
        assert!(c.polys.num_cells() >= 8);
    }
    #[test]
    fn test_8() {
        let c = compass_rose(2.0, 8);
        assert!(c.polys.num_cells() >= 16);
    }
}
