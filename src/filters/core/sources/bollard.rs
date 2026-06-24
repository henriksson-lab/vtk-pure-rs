//! Bollard (short post) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn bollard(radius: f64, height: f64, cap_radius: f64, resolution: usize) -> PolyData {
    let res = resolution.max(6);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Shaft
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
    }
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([radius * a.cos(), radius * a.sin(), height * 0.85]);
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Dome cap
    let cap_rings = 4;
    for ir in 1..=cap_rings {
        let t = ir as f64 / cap_rings as f64;
        let a = t * std::f64::consts::FRAC_PI_2;
        let cr = cap_radius * (a.cos());
        let cz = height * 0.85 + cap_radius * (a.sin());
        for i in 0..res {
            let ang = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([cr * ang.cos(), cr * ang.sin(), cz]);
        }
    }
    let base = res;
    for ir in 0..cap_rings {
        let _r0 = if ir == 0 { 0 } else { res + (ir - 1) * res };
        let r1 = res + ir * res;
        let ring0_base = if ir == 0 {
            base
        } else {
            res + base + (ir - 1) * res
        };
        for i in 0..res {
            let j = (i + 1) % res;
            let a0 = if ir == 0 { base + i } else { ring0_base + i };
            let a1 = if ir == 0 { base + j } else { ring0_base + j };
            polys.push_cell(&[
                a0 as i64,
                a1 as i64,
                (r1 + j + res) as i64,
                (r1 + i + res) as i64,
            ]);
        }
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
        let b = bollard(0.15, 1.0, 0.2, 8);
        assert!(b.points.len() > 20);
    }
}
