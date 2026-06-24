//! Satellite with solar panels and dish antenna.
use crate::data::{CellArray, Points, PolyData};
pub fn satellite(
    body_size: f64,
    panel_w: f64,
    panel_h: f64,
    dish_r: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let bs = body_size / 2.0;
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
    // Body
    ab(&mut pts, &mut polys, -bs, -bs, -bs, bs, bs, bs);
    // Solar panels
    for &side in &[-1.0f64, 1.0] {
        let px = side * (bs + panel_w / 2.0 + bs * 0.2);
        let pb = pts.len();
        pts.push([px - panel_w / 2.0, -panel_h / 2.0, 0.0]);
        pts.push([px + panel_w / 2.0, -panel_h / 2.0, 0.0]);
        pts.push([px + panel_w / 2.0, panel_h / 2.0, 0.0]);
        pts.push([px - panel_w / 2.0, panel_h / 2.0, 0.0]);
        polys.push_cell(&[pb as i64, (pb + 1) as i64, (pb + 2) as i64, (pb + 3) as i64]);
        // Panel arm
        let ab2 = pts.len();
        pts.push([side * bs, 0.0, 0.0]);
        pts.push([px - side * panel_w / 2.0, 0.0, 0.0]);
        lines.push_cell(&[ab2 as i64, (ab2 + 1) as i64]);
    }
    // Dish antenna
    let db = pts.len();
    let vres = res / 2;
    for iv in 0..=vres {
        let t = iv as f64 / vres as f64;
        let r = dish_r * t;
        let z = bs + t * t * dish_r * 0.5;
        for iu in 0..=res {
            let a = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    let dw = res + 1;
    for iv in 0..vres {
        for iu in 0..res {
            polys.push_cell(&[
                (db + iv * dw + iu) as i64,
                (db + iv * dw + iu + 1) as i64,
                (db + (iv + 1) * dw + iu + 1) as i64,
                (db + (iv + 1) * dw + iu) as i64,
            ]);
        }
    }
    // Antenna feed arm
    let fb = pts.len();
    pts.push([0.0, 0.0, bs]);
    pts.push([0.0, 0.0, bs + dish_r * 1.5]);
    lines.push_cell(&[fb as i64, (fb + 1) as i64]);
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
        let s = satellite(1.0, 3.0, 1.5, 0.5, 8);
        assert!(s.polys.num_cells() > 15);
        assert!(s.lines.num_cells() >= 3);
    }
}
