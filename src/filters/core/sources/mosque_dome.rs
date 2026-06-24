//! Mosque dome (onion dome) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn onion_dome(
    radius: f64,
    height: f64,
    neck_radius: f64,
    neck_height: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let vres = 12;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iv in 0..=vres {
        let t = iv as f64 / vres as f64;
        let (r, z) = if t < 0.3 {
            // Neck section
            let lt = t / 0.3;
            (neck_radius, lt * neck_height)
        } else {
            // Onion bulb
            let lt = (t - 0.3) / 0.7;
            let bulge = (lt * std::f64::consts::PI).sin();
            let r = neck_radius + (radius - neck_radius) * bulge * (1.0 - lt * 0.3);
            let z = neck_height + lt * (height - neck_height);
            (r, z)
        };
        for iu in 0..res {
            let a = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for iv in 0..vres {
        for iu in 0..res {
            let iu1 = (iu + 1) % res;
            polys.push_cell(&[
                (iv * res + iu) as i64,
                (iv * res + iu1) as i64,
                ((iv + 1) * res + iu1) as i64,
                ((iv + 1) * res + iu) as i64,
            ]);
        }
    }
    // Finial (spike on top)
    let fc = pts.len();
    pts.push([0.0, 0.0, height + radius * 0.3]);
    let top = vres * res;
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[fc as i64, (top + i) as i64, (top + j) as i64]);
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
        let d = onion_dome(3.0, 6.0, 1.5, 2.0, 12);
        assert!(d.polys.num_cells() > 100);
    }
}
