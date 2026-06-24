//! Submarine geometry (hull + conning tower + propeller + dive planes).
use crate::data::{CellArray, Points, PolyData};
pub fn submarine(hull_l: f64, hull_r: f64, tower_h: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let hl = hull_l / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Hull (tapered cylinder)
    let nseg = 10;
    for is in 0..=nseg {
        let t = is as f64 / nseg as f64;
        let x = -hl + hull_l * t;
        let taper = 1.0 - (2.0 * t - 1.0).powi(2);
        let r = hull_r * taper.max(0.05).sqrt();
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
    // Conning tower
    let tw = hull_l * 0.08;
    let td = hull_r * 0.4;
    let cb = pts.len();
    pts.push([-tw, -td, hull_r]);
    pts.push([tw, -td, hull_r]);
    pts.push([tw, td, hull_r]);
    pts.push([-tw, td, hull_r]);
    pts.push([-tw, -td, hull_r + tower_h]);
    pts.push([tw, -td, hull_r + tower_h]);
    pts.push([tw, td, hull_r + tower_h]);
    pts.push([-tw, td, hull_r + tower_h]);
    let f = |i: usize| (cb + i) as i64;
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    polys.push_cell(&[f(3), f(0), f(4), f(7)]);
    polys.push_cell(&[f(4), f(5), f(6), f(7)]); // top
                                                // Propeller (lines at stern)
    let prop_x = hl * 0.95;
    let pb = pts.len();
    pts.push([prop_x, 0.0, 0.0]);
    for bi in 0..4 {
        let a = std::f64::consts::FRAC_PI_2 * bi as f64;
        let pr = hull_r * 0.6;
        let ti = pts.len();
        pts.push([prop_x, pr * a.cos(), pr * a.sin()]);
        lines.push_cell(&[pb as i64, ti as i64]);
    }
    // Dive planes
    let dp_x = -hl * 0.3;
    let dp_span = hull_r * 1.5;
    for &side in &[-1.0f64, 1.0] {
        let db = pts.len();
        pts.push([dp_x, side * hull_r, 0.0]);
        pts.push([dp_x, side * dp_span, 0.0]);
        lines.push_cell(&[db as i64, (db + 1) as i64]);
    }
    // Rudder
    let rb = pts.len();
    pts.push([hl * 0.8, 0.0, -hull_r * 0.2]);
    pts.push([hl * 0.8, 0.0, hull_r * 0.8]);
    lines.push_cell(&[rb as i64, (rb + 1) as i64]);
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
        let s = submarine(20.0, 2.0, 2.5, 10);
        assert!(s.polys.num_cells() > 50);
        assert!(s.lines.num_cells() > 5);
    }
}
