//! Dome (hemisphere) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn dome(radius: f64, u_res: usize, v_res: usize) -> PolyData {
    let ur = u_res.max(3);
    let vr = v_res.max(2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Only upper hemisphere (theta from 0 to pi/2)
    for iv in 0..=vr {
        let v = std::f64::consts::FRAC_PI_2 * iv as f64 / vr as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=ur {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / ur as f64;
            pts.push([radius * sv * u.cos(), radius * sv * u.sin(), radius * cv]);
        }
    }
    let w = ur + 1;
    for iv in 0..vr {
        for iu in 0..ur {
            polys.push_cell(&[
                (iv * w + iu) as i64,
                (iv * w + iu + 1) as i64,
                ((iv + 1) * w + iu + 1) as i64,
                ((iv + 1) * w + iu) as i64,
            ]);
        }
    }
    // Base cap
    let base = vr * w;
    let bc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for iu in 0..ur {
        polys.push_cell(&[bc as i64, (base + iu + 1) as i64, (base + iu) as i64]);
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
    fn test() {
        let d = dome(1.0, 16, 8);
        assert!(d.points.len() > 50);
        assert!(d.polys.num_cells() > 50);
    }
}
