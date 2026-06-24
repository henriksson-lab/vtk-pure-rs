//! Agricultural silo (cylinder + dome top).
use crate::data::{CellArray, Points, PolyData};
pub fn silo(radius: f64, height: f64, dome_height: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Cylinder
    for ring in 0..=1 {
        let z = if ring == 0 { 0.0 } else { height };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([radius * a.cos(), radius * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Dome
    let dome_rings = 4;
    for dr in 1..=dome_rings {
        let t = dr as f64 / dome_rings as f64;
        let a = t * std::f64::consts::FRAC_PI_2;
        let r = radius * a.cos();
        let z = height + dome_height * a.sin();
        for i in 0..res {
            let ang = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([r * ang.cos(), r * ang.sin(), z]);
        }
    }
    let base = res;
    for dr in 0..dome_rings {
        let r0 = if dr == 0 {
            base
        } else {
            base + res + (dr - 1) * res
        };
        let r1 = base + res + dr * res;
        if dr == 0 {
            for i in 0..res {
                let j = (i + 1) % res;
                polys.push_cell(&[
                    (base + i) as i64,
                    (base + j) as i64,
                    (r1 + j) as i64,
                    (r1 + i) as i64,
                ]);
            }
        } else {
            for i in 0..res {
                let j = (i + 1) % res;
                polys.push_cell(&[
                    (r0 + i) as i64,
                    (r0 + j) as i64,
                    (r1 + j) as i64,
                    (r1 + i) as i64,
                ]);
            }
        }
    }
    // Top cap
    let tc = pts.len();
    pts.push([0.0, 0.0, height + dome_height]);
    let top_ring = base + res + (dome_rings - 1) * res;
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[tc as i64, (top_ring + i) as i64, (top_ring + j) as i64]);
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
        let s = silo(2.0, 8.0, 1.5, 12);
        assert!(s.polys.num_cells() > 40);
    }
}
