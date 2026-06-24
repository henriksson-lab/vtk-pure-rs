//! Rocket geometry (body + nose cone + fins + nozzle).
use crate::data::{CellArray, Points, PolyData};
pub fn rocket(
    body_r: f64,
    body_h: f64,
    nose_h: f64,
    fin_h: f64,
    fin_span: f64,
    num_fins: usize,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let nf = num_fins.max(2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Body cylinder
    for ring in 0..=1 {
        let z = if ring == 0 { 0.0 } else { body_h };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([body_r * a.cos(), body_r * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Nose cone
    let nc = pts.len();
    pts.push([0.0, 0.0, body_h + nose_h]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[nc as i64, (res + i) as i64, (res + j) as i64]);
    }
    // Nozzle (smaller cylinder at bottom)
    let nozzle_r = body_r * 0.6;
    let nozzle_h = body_h * 0.1;
    let nb = pts.len();
    for ring in 0..=1 {
        let z = if ring == 0 { -nozzle_h } else { 0.0 };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([nozzle_r * a.cos(), nozzle_r * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (nb + i) as i64,
            (nb + j) as i64,
            (nb + res + j) as i64,
            (nb + res + i) as i64,
        ]);
    }
    // Fins
    for fi in 0..nf {
        let a = 2.0 * std::f64::consts::PI * fi as f64 / nf as f64;
        let ca = a.cos();
        let sa = a.sin();
        let fb = pts.len();
        pts.push([body_r * ca, body_r * sa, 0.0]);
        pts.push([
            (body_r + fin_span) * ca,
            (body_r + fin_span) * sa,
            fin_h * 0.3,
        ]);
        pts.push([body_r * ca, body_r * sa, fin_h]);
        polys.push_cell(&[fb as i64, (fb + 1) as i64, (fb + 2) as i64]);
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
        let r = rocket(0.5, 5.0, 1.5, 1.5, 0.8, 4, 12);
        assert!(r.polys.num_cells() > 30);
    }
}
