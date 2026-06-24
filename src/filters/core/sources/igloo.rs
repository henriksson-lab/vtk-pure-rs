//! Igloo (hemispherical dome with entrance tunnel).
use crate::data::{CellArray, Points, PolyData};
pub fn igloo(
    radius: f64,
    entrance_w: f64,
    entrance_h: f64,
    entrance_d: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let vres = res / 2;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Dome
    for iv in 0..=vres {
        let v = std::f64::consts::FRAC_PI_2 * iv as f64 / vres as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            pts.push([radius * sv * u.cos(), radius * sv * u.sin(), radius * cv]);
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
    // Entrance tunnel
    let hw = entrance_w / 2.0;
    let eh = entrance_h;
    let tb = pts.len();
    pts.push([-hw, -radius, 0.0]);
    pts.push([hw, -radius, 0.0]);
    pts.push([hw, -radius, eh]);
    pts.push([-hw, -radius, eh]);
    pts.push([-hw, -radius - entrance_d, 0.0]);
    pts.push([hw, -radius - entrance_d, 0.0]);
    pts.push([hw, -radius - entrance_d, eh]);
    pts.push([-hw, -radius - entrance_d, eh]);
    let f = |i: usize| (tb + i) as i64;
    polys.push_cell(&[f(4), f(5), f(6), f(7)]); // front
    polys.push_cell(&[f(0), f(4), f(7), f(3)]); // left wall
    polys.push_cell(&[f(1), f(2), f(6), f(5)]); // right wall
    polys.push_cell(&[f(3), f(7), f(6), f(2)]); // ceiling
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let i = igloo(3.0, 1.5, 1.5, 2.0, 12);
        assert!(i.polys.num_cells() > 30);
    }
}
