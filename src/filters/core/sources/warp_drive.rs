//! Alcubierre warp drive bubble geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn warp_bubble(bubble_r: f64, ship_r: f64, wall_thickness: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Outer bubble (oblate spheroid)
    let vres = res;
    let stretch = 1.5;
    for iv in 0..=vres {
        let v = std::f64::consts::PI * iv as f64 / vres as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            pts.push([
                bubble_r * stretch * sv * u.cos(),
                bubble_r * sv * u.sin(),
                bubble_r * cv,
            ]);
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
    // Inner bubble (slightly smaller)
    let ir = bubble_r - wall_thickness;
    let ib = pts.len();
    for iv in 0..=vres {
        let v = std::f64::consts::PI * iv as f64 / vres as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            pts.push([ir * stretch * sv * u.cos(), ir * sv * u.sin(), ir * cv]);
        }
    }
    for iv in 0..vres {
        for iu in 0..res {
            polys.push_cell(&[
                (ib + (iv + 1) * w + iu) as i64,
                (ib + (iv + 1) * w + iu + 1) as i64,
                (ib + iv * w + iu + 1) as i64,
                (ib + iv * w + iu) as i64,
            ]);
        }
    }
    // Ship (small box at center)
    let sb = pts.len();
    let sr = ship_r;
    pts.push([-sr * 2.0, -sr, -sr]);
    pts.push([sr * 2.0, -sr, -sr]);
    pts.push([sr * 2.0, sr, -sr]);
    pts.push([-sr * 2.0, sr, -sr]);
    pts.push([-sr * 2.0, -sr, sr]);
    pts.push([sr * 2.0, -sr, sr]);
    pts.push([sr * 2.0, sr, sr]);
    pts.push([-sr * 2.0, sr, sr]);
    let f = |i: usize| (sb + i) as i64;
    polys.push_cell(&[f(0), f(3), f(2), f(1)]);
    polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    polys.push_cell(&[f(0), f(4), f(7), f(3)]);
    polys.push_cell(&[f(1), f(2), f(6), f(5)]);
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
        let w = warp_bubble(10.0, 1.0, 0.5, 8);
        assert!(w.polys.num_cells() > 100);
    }
}
