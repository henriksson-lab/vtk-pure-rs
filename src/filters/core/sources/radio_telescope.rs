//! Radio telescope array (multiple dishes on a baseline).
use crate::data::{CellArray, Points, PolyData};
pub fn radio_telescope_array(
    dish_r: f64,
    dish_depth: f64,
    num_dishes: usize,
    baseline: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let nd = num_dishes.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let vres = res / 2;
    for di in 0..nd {
        let dx = if nd > 1 {
            -baseline / 2.0 + baseline * di as f64 / (nd - 1) as f64
        } else {
            0.0
        };
        let f = dish_r * dish_r / (4.0 * dish_depth);
        // Dish surface
        let db = pts.len();
        for iv in 0..=vres {
            let r = dish_r * iv as f64 / vres as f64;
            let z = r * r / (4.0 * f);
            for iu in 0..=res {
                let a = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
                pts.push([dx + r * a.cos(), r * a.sin(), z + dish_r * 0.5]);
            }
        }
        let w = res + 1;
        for iv in 0..vres {
            for iu in 0..res {
                polys.push_cell(&[
                    (db + iv * w + iu) as i64,
                    (db + iv * w + iu + 1) as i64,
                    (db + (iv + 1) * w + iu + 1) as i64,
                    (db + (iv + 1) * w + iu) as i64,
                ]);
            }
        }
        // Feed arm
        let fb = pts.len();
        pts.push([dx, 0.0, f + dish_r * 0.5]);
        for i in 0..3 {
            let a = 2.0 * std::f64::consts::PI * i as f64 / 3.0;
            let ei = pts.len();
            pts.push([
                dx + dish_r * 0.85 * a.cos(),
                dish_r * 0.85 * a.sin(),
                dish_r * 0.5 + dish_depth * 0.85,
            ]);
            lines.push_cell(&[fb as i64, ei as i64]);
        }
        // Mount pedestal
        let mb = pts.len();
        pts.push([dx, 0.0, 0.0]);
        pts.push([dx, 0.0, dish_r * 0.5]);
        lines.push_cell(&[mb as i64, (mb + 1) as i64]);
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
    fn test_single() {
        let r = radio_telescope_array(5.0, 1.5, 1, 0.0, 12);
        assert!(r.polys.num_cells() > 20);
    }
    #[test]
    fn test_array() {
        let r = radio_telescope_array(3.0, 1.0, 4, 100.0, 8);
        assert!(r.polys.num_cells() > 40);
    }
}
