//! Multi-level parking garage geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn parking_garage(
    width: f64,
    depth: f64,
    levels: usize,
    level_h: f64,
    ramp_w: f64,
) -> PolyData {
    let nl = levels.max(1);
    let hw = width / 2.0;
    let hd = depth / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let ab = |pts: &mut Points<f64>,
              polys: &mut CellArray,
              x0: f64,
              y0: f64,
              z0: f64,
              x1: f64,
              y1: f64,
              z1: f64| {
        let b = pts.len();
        pts.push([x0, y0, z0]);
        pts.push([x1, y0, z0]);
        pts.push([x1, y1, z0]);
        pts.push([x0, y1, z0]);
        pts.push([x0, y0, z1]);
        pts.push([x1, y0, z1]);
        pts.push([x1, y1, z1]);
        pts.push([x0, y1, z1]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
        polys.push_cell(&[f(0), f(4), f(7), f(3)]);
        polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    };
    // Floor slabs
    for li in 0..=nl {
        let z = li as f64 * level_h;
        ab(&mut pts, &mut polys, -hw, -hd, z, hw, hd, z + 0.2);
    }
    // Corner columns
    let col_r = width * 0.03;
    for &x in &[-hw + col_r, hw - col_r] {
        for &y in &[-hd + col_r, hd - col_r] {
            for li in 0..nl {
                let z = li as f64 * level_h;
                let cb = pts.len();
                pts.push([x, y, z]);
                pts.push([x, y, z + level_h]);
                lines.push_cell(&[cb as i64, (cb + 1) as i64]);
            }
        }
    }
    // Ramps between levels
    for li in 0..nl {
        let z0 = li as f64 * level_h + 0.2;
        let z1 = (li + 1) as f64 * level_h;
        let side = if li % 2 == 0 { -1.0 } else { 1.0 };
        let rb = pts.len();
        pts.push([side * (hw - ramp_w), -hd, z0]);
        pts.push([side * hw, -hd, z0]);
        pts.push([side * hw, hd, z1]);
        pts.push([side * (hw - ramp_w), hd, z1]);
        polys.push_cell(&[rb as i64, (rb + 1) as i64, (rb + 2) as i64, (rb + 3) as i64]);
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
        let g = parking_garage(20.0, 30.0, 3, 3.0, 4.0);
        assert!(g.polys.num_cells() > 5);
        assert!(g.lines.num_cells() > 5);
    }
}
