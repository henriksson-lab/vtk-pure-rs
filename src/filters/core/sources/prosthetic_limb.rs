//! Prosthetic limb (below-knee prosthesis).
use crate::data::{CellArray, Points, PolyData};
pub fn below_knee_prosthesis(
    socket_r: f64,
    socket_h: f64,
    pylon_r: f64,
    pylon_h: f64,
    foot_l: f64,
    foot_w: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let lines = CellArray::new();
    // Socket (truncated cone)
    for ring in 0..=1 {
        let t = ring as f64;
        let r = socket_r * (1.0 - t * 0.2);
        let z = socket_h + pylon_h + if ring == 0 { 0.0 } else { socket_h };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Pylon (thin cylinder)
    let pb = pts.len();
    for ring in 0..=1 {
        let z = if ring == 0 { 0.15 } else { socket_h + pylon_h };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([pylon_r * a.cos(), pylon_r * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (pb + i) as i64,
            (pb + j) as i64,
            (pb + res + j) as i64,
            (pb + res + i) as i64,
        ]);
    }
    // Foot
    let hw = foot_w / 2.0;
    let hl = foot_l / 2.0;
    let fh = 0.15;
    let fb = pts.len();
    pts.push([-hl, -hw, 0.0]);
    pts.push([hl, -hw, 0.0]);
    pts.push([hl, hw, 0.0]);
    pts.push([-hl, hw, 0.0]);
    pts.push([-hl, -hw, fh]);
    pts.push([hl, -hw, fh]);
    pts.push([hl, hw, fh]);
    pts.push([-hl, hw, fh]);
    let f = |i: usize| (fb + i) as i64;
    polys.push_cell(&[f(0), f(3), f(2), f(1)]);
    polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    polys.push_cell(&[f(0), f(4), f(7), f(3)]);
    polys.push_cell(&[f(1), f(2), f(6), f(5)]);
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
        let p = below_knee_prosthesis(0.06, 0.15, 0.015, 0.3, 0.25, 0.08, 8);
        assert!(p.polys.num_cells() > 15);
    }
}
