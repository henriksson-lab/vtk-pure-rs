//! Mitochondria geometry (double membrane with cristae).
use crate::data::{CellArray, Points, PolyData};
pub fn mitochondria(length: f64, radius: f64, num_cristae: usize, resolution: usize) -> PolyData {
    let res = resolution.max(6);
    let hl = length / 2.0;
    let nc = num_cristae.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Outer membrane (capsule shape)
    let nseg = 8;
    for is in 0..=nseg {
        let t = is as f64 / nseg as f64;
        let x = -hl + length * t;
        let taper = (1.0 - (2.0 * t - 1.0).powi(4)).max(0.05).sqrt();
        let r = radius * taper;
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, r * a.cos(), r * a.sin()]);
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
    // Inner membrane (slightly smaller)
    let ir = radius * 0.85;
    let ib = pts.len();
    for is in 0..=nseg {
        let t = is as f64 / nseg as f64;
        let x = -hl + length * t;
        let taper = (1.0 - (2.0 * t - 1.0).powi(4)).max(0.05).sqrt();
        let r = ir * taper;
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, r * a.cos(), r * a.sin()]);
        }
    }
    for is in 0..nseg {
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (ib + is * res + j) as i64,
                (ib + is * res + i) as i64,
                (ib + (is + 1) * res + i) as i64,
                (ib + (is + 1) * res + j) as i64,
            ]);
        }
    }
    // Cristae (folds of inner membrane)
    for ci in 0..nc {
        let cx = -hl * 0.6 + ci as f64 * length * 0.6 / (nc as f64);
        let fold_h = ir * 0.6;
        let fold_w = length * 0.03;
        let fb = pts.len();
        pts.push([cx, -ir * 0.3, -fold_h]);
        pts.push([cx + fold_w, -ir * 0.3, -fold_h]);
        pts.push([cx + fold_w, ir * 0.3, fold_h]);
        pts.push([cx, ir * 0.3, fold_h]);
        polys.push_cell(&[fb as i64, (fb + 1) as i64, (fb + 2) as i64, (fb + 3) as i64]);
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
        let m = mitochondria(4.0, 1.0, 5, 8);
        assert!(m.polys.num_cells() > 50);
    }
}
