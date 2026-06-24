//! Observatory dome with slit opening.
use crate::data::{CellArray, Points, PolyData};
pub fn observatory_dome(
    radius: f64,
    slit_width_degrees: f64,
    u_res: usize,
    v_res: usize,
) -> PolyData {
    let ur = u_res.max(8);
    let vr = v_res.max(4);
    let slit_half = (slit_width_degrees / 2.0).to_radians();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
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
            let u_angle = 2.0 * std::f64::consts::PI * iu as f64 / ur as f64;
            // Skip slit region
            if u_angle.abs() < slit_half || (2.0 * std::f64::consts::PI - u_angle).abs() < slit_half
            {
                continue;
            }
            polys.push_cell(&[
                (iv * w + iu) as i64,
                (iv * w + iu + 1) as i64,
                ((iv + 1) * w + iu + 1) as i64,
                ((iv + 1) * w + iu) as i64,
            ]);
        }
    }
    // Cylindrical base
    let base = pts.len();
    let base_h = radius * 0.3;
    for i in 0..ur {
        let a = 2.0 * std::f64::consts::PI * i as f64 / ur as f64;
        pts.push([radius * a.cos(), radius * a.sin(), -base_h]);
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
    }
    for i in 0..ur {
        let j = (i + 1) % ur;
        polys.push_cell(&[
            (base + i * 2) as i64,
            (base + j * 2) as i64,
            (base + j * 2 + 1) as i64,
            (base + i * 2 + 1) as i64,
        ]);
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
        let d = observatory_dome(5.0, 20.0, 24, 8);
        assert!(d.polys.num_cells() > 50);
    }
}
