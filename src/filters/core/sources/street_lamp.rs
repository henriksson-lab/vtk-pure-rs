//! Street lamp with curved arm and shade.
use crate::data::{CellArray, Points, PolyData};
pub fn street_lamp(pole_h: f64, arm_reach: f64, shade_r: f64, resolution: usize) -> PolyData {
    let res = resolution.max(6);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Pole
    let pr = pole_h * 0.02;
    for ring in 0..=1 {
        let z = if ring == 0 { 0.0 } else { pole_h };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([pr * a.cos(), pr * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Curved arm
    let arm_steps = 8;
    let mut arm_ids = Vec::new();
    for i in 0..=arm_steps {
        let t = i as f64 / arm_steps as f64;
        let x = arm_reach * t;
        let z = pole_h - arm_reach * 0.15 * (1.0 - (1.0 - t).powi(2));
        let idx = pts.len();
        pts.push([x, 0.0, z]);
        arm_ids.push(idx as i64);
    }
    lines.push_cell(&arm_ids);
    // Shade (inverted cone)
    let shade_z = pole_h - arm_reach * 0.15;
    let sc = pts.len();
    pts.push([arm_reach, 0.0, shade_z + shade_r * 0.3]);
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([arm_reach + shade_r * a.cos(), shade_r * a.sin(), shade_z]);
    }
    for i in 0..res {
        let j = if i + 1 < res { sc + 2 + i } else { sc + 1 };
        polys.push_cell(&[sc as i64, (sc + 1 + i) as i64, j as i64]);
    }
    // Base plate
    let bp = pts.len();
    let br = pr * 4.0;
    pts.push([-br, -br, 0.0]);
    pts.push([br, -br, 0.0]);
    pts.push([br, br, 0.0]);
    pts.push([-br, br, 0.0]);
    polys.push_cell(&[bp as i64, (bp + 1) as i64, (bp + 2) as i64, (bp + 3) as i64]);
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
        let l = street_lamp(5.0, 1.5, 0.3, 8);
        assert!(l.polys.num_cells() > 10);
    }
}
