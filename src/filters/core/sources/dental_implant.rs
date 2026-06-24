//! Dental implant (screw + abutment + crown).
use crate::data::{CellArray, Points, PolyData};
pub fn dental_implant(
    implant_r: f64,
    implant_l: f64,
    abutment_h: f64,
    crown_r: f64,
    crown_h: f64,
    thread_count: usize,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let nt = thread_count.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let lines = CellArray::new();
    // Implant screw (tapered cylinder with thread grooves)
    let nseg = 8;
    for is in 0..=nseg {
        let t = is as f64 / nseg as f64;
        let z = -t * implant_l;
        let taper = 1.0 - t * 0.15;
        let r = implant_r * taper;
        // Thread grooves
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            let thread_mod = 1.0 + 0.08 * (a * nt as f64 + z * 20.0).sin();
            pts.push([r * thread_mod * a.cos(), r * thread_mod * a.sin(), z]);
        }
    }
    for is in 0..nseg {
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (is * res + i) as i64,
                (is * res + j) as i64,
                ((is + 1) * res + j) as i64,
                ((is + 1) * res + i) as i64,
            ]);
        }
    }
    // Tip (cone at bottom)
    let tc = pts.len();
    pts.push([0.0, 0.0, -implant_l - implant_r * 0.5]);
    let tip_ring = nseg * res;
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[tc as i64, (tip_ring + j) as i64, (tip_ring + i) as i64]);
    }
    // Abutment (narrower cylinder on top)
    let ar = implant_r * 0.6;
    let ab = pts.len();
    for ring in 0..=1 {
        let z = if ring == 0 { 0.0 } else { abutment_h };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([ar * a.cos(), ar * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (ab + i) as i64,
            (ab + j) as i64,
            (ab + res + j) as i64,
            (ab + res + i) as i64,
        ]);
    }
    // Crown (dome shape)
    let cb = pts.len();
    let dome_rings = 4;
    for dr in 0..=dome_rings {
        let t = dr as f64 / dome_rings as f64;
        let a = t * std::f64::consts::FRAC_PI_2;
        let r = crown_r * a.cos();
        let z = abutment_h + crown_h * a.sin();
        for i in 0..res {
            let ang = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([r * ang.cos(), r * ang.sin(), z]);
        }
    }
    for dr in 0..dome_rings {
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (cb + dr * res + i) as i64,
                (cb + dr * res + j) as i64,
                (cb + (dr + 1) * res + j) as i64,
                (cb + (dr + 1) * res + i) as i64,
            ]);
        }
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
        let d = dental_implant(0.002, 0.01, 0.005, 0.004, 0.008, 5, 8);
        assert!(d.polys.num_cells() > 40);
    }
}
