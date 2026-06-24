//! Helicopter geometry (fuselage + main rotor + tail rotor + tail boom).
use crate::data::{CellArray, Points, PolyData};
pub fn helicopter(
    fuselage_l: f64,
    fuselage_r: f64,
    rotor_r: f64,
    tail_l: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let hl = fuselage_l / 2.0;
    // Fuselage (ellipsoid-ish cylinder)
    let nseg = 6;
    for is in 0..=nseg {
        let t = is as f64 / nseg as f64;
        let x = -hl + fuselage_l * t;
        let r = fuselage_r * (1.0 - (2.0 * t - 1.0).powi(2)).max(0.1).sqrt();
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, r * a.cos(), r * a.sin() + fuselage_r]);
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
    // Main rotor (lines from hub)
    let hub_z = fuselage_r * 2.2;
    let hb = pts.len();
    pts.push([0.0, 0.0, hub_z]);
    for bi in 0..4 {
        let a = std::f64::consts::FRAC_PI_2 * bi as f64;
        let ti = pts.len();
        pts.push([rotor_r * a.cos(), rotor_r * a.sin(), hub_z]);
        lines.push_cell(&[hb as i64, ti as i64]);
    }
    // Mast
    let mb = pts.len();
    pts.push([0.0, 0.0, fuselage_r * 2.0]);
    pts.push([0.0, 0.0, hub_z]);
    lines.push_cell(&[mb as i64, (mb + 1) as i64]);
    // Tail boom
    let tb = pts.len();
    pts.push([hl, 0.0, fuselage_r]);
    pts.push([hl + tail_l, 0.0, fuselage_r * 1.5]);
    lines.push_cell(&[tb as i64, (tb + 1) as i64]);
    // Tail rotor
    let tr_center = pts.len();
    pts.push([hl + tail_l, 0.0, fuselage_r * 1.5]);
    for bi in 0..2 {
        let a = std::f64::consts::PI * bi as f64;
        let tr = fuselage_r * 0.8;
        let ti = pts.len();
        pts.push([hl + tail_l, tr * a.cos(), fuselage_r * 1.5 + tr * a.sin()]);
        lines.push_cell(&[tr_center as i64, ti as i64]);
    }
    // Tail fin
    let fb = pts.len();
    pts.push([hl + tail_l * 0.7, 0.0, fuselage_r * 1.3]);
    pts.push([hl + tail_l, 0.0, fuselage_r * 2.2]);
    pts.push([hl + tail_l, 0.0, fuselage_r * 1.5]);
    polys.push_cell(&[fb as i64, (fb + 1) as i64, (fb + 2) as i64]);
    // Skids
    for &y in &[-fuselage_r * 0.6, fuselage_r * 0.6] {
        let sb = pts.len();
        pts.push([-hl * 0.8, y, 0.0]);
        pts.push([hl * 0.5, y, 0.0]);
        lines.push_cell(&[sb as i64, (sb + 1) as i64]);
        // Struts
        let s1 = pts.len();
        pts.push([-hl * 0.3, y, 0.0]);
        pts.push([-hl * 0.3, y, fuselage_r * 0.5]);
        lines.push_cell(&[s1 as i64, (s1 + 1) as i64]);
        let s2 = pts.len();
        pts.push([hl * 0.2, y, 0.0]);
        pts.push([hl * 0.2, y, fuselage_r * 0.5]);
        lines.push_cell(&[s2 as i64, (s2 + 1) as i64]);
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
        let h = helicopter(6.0, 1.0, 5.0, 4.0, 8);
        assert!(h.polys.num_cells() > 20);
        assert!(h.lines.num_cells() > 10);
    }
}
