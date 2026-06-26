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
    let cap_base = pts.len();
    for ir in 0..cap_rings {
        let t = ir as f64 / cap_rings as f64;
        let a = t * std::f64::consts::FRAC_PI_2;
        let cr = cap_radius * (a.cos());
        let cz = height * 0.85 + cap_radius * (a.sin());
        for i in 0..res {
            let ang = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([cr * ang.cos(), cr * ang.sin(), cz]);
        }
    }
    let pole = pts.len();
    pts.push([0.0, 0.0, height * 0.85 + cap_radius]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (res + i) as i64,
            (res + j) as i64,
            (cap_base + j) as i64,
            (cap_base + i) as i64,
        ]);
    }
    for ir in 0..cap_rings {
        for i in 0..res {
            let j = (i + 1) % res;
            let r0 = cap_base + ir * res;
            if ir + 1 == cap_rings {
                polys.push_cell(&[(r0 + i) as i64, (r0 + j) as i64, pole as i64]);
            } else {
                let r1 = cap_base + (ir + 1) * res;
                polys.push_cell(&[
                    (r0 + i) as i64,
                    (r0 + j) as i64,
                    (r1 + j) as i64,
                    (r1 + i) as i64,
                ]);
            }
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
