//! Satellite dish with receiver arm.
use crate::data::{CellArray, Points, PolyData};
pub fn satellite_dish(dish_r: f64, depth: f64, pole_h: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let vres = res / 2;
    let f = dish_r * dish_r / (4.0 * depth);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Dish surface
    for iv in 0..=vres {
        let r = dish_r * iv as f64 / vres as f64;
        let z = pole_h + r * r / (4.0 * f);
        for iu in 0..=res {
            let a = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    let w = res + 1;
    for iv in 0..vres {
        for iu in 0..res {
            polys.push_cell(&[
                (iv * w + iu) as i64,
                (iv * w + iu + 1) as i64,
                ((iv + 1) * w + iu + 1) as i64,
                ((iv + 1) * w + iu) as i64,
            ]);
        }
    }
    // Pole
    let pb = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    pts.push([0.0, 0.0, pole_h]);
    lines.push_cell(&[pb as i64, (pb + 1) as i64]);
    // Feed arm
    let feed_z = pole_h + f;
    let fb = pts.len();
    pts.push([0.0, 0.0, feed_z]);
    for i in 0..3 {
        let a = 2.0 * std::f64::consts::PI * i as f64 / 3.0;
        let edge_idx = pts.len();
        pts.push([
            dish_r * 0.85 * a.cos(),
            dish_r * 0.85 * a.sin(),
            pole_h + dish_r * 0.85 * dish_r * 0.85 / (4.0 * f),
        ]);
        lines.push_cell(&[fb as i64, edge_idx as i64]);
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
        let d = satellite_dish(1.5, 0.5, 2.0, 12);
        assert!(d.polys.num_cells() > 20);
        assert!(d.lines.num_cells() >= 4);
    }
}
