//! Water wheel (paddlewheel) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn water_wheel(radius: f64, width: f64, num_paddles: usize, paddle_height: f64) -> PolyData {
    let np = num_paddles.max(4);
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Hub (two circles)
    let hub_r = radius * 0.15;
    let res = np * 2;
    for side in [-hw, hw] {
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([hub_r * a.cos(), side, hub_r * a.sin()]);
        }
    }
    // Axle
    let ab = pts.len();
    pts.push([0.0, -hw * 1.5, 0.0]);
    pts.push([0.0, hw * 1.5, 0.0]);
    lines.push_cell(&[ab as i64, (ab + 1) as i64]);
    // Paddles
    for i in 0..np {
        let a = 2.0 * std::f64::consts::PI * i as f64 / np as f64;
        let ca = a.cos();
        let sa = a.sin();
        let inner_r = radius - paddle_height;
        let b = pts.len();
        pts.push([inner_r * ca, -hw, inner_r * sa]);
        pts.push([radius * ca, -hw, radius * sa]);
        pts.push([radius * ca, hw, radius * sa]);
        pts.push([inner_r * ca, hw, inner_r * sa]);
        polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64, (b + 3) as i64]);
        // Spoke
        let sb = pts.len();
        pts.push([hub_r * ca, 0.0, hub_r * sa]);
        pts.push([inner_r * ca, 0.0, inner_r * sa]);
        lines.push_cell(&[sb as i64, (sb + 1) as i64]);
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
        let w = water_wheel(3.0, 1.0, 8, 0.8);
        assert!(w.polys.num_cells() >= 8);
        assert!(w.lines.num_cells() >= 8);
    }
}
