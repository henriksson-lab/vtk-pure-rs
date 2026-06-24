//! Street lamp post geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn lamp_post(
    height: f64,
    pole_radius: f64,
    arm_length: f64,
    shade_radius: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Pole
    for ring in 0..=1 {
        let z = if ring == 0 { 0.0 } else { height };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([pole_radius * a.cos(), pole_radius * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Arm (curved)
    let arm_steps = 8;
    let mut arm_ids = Vec::new();
    for i in 0..=arm_steps {
        let t = i as f64 / arm_steps as f64;
        let x = arm_length * t;
        let z = height - arm_length * 0.3 * (1.0 - (1.0 - t).powi(2));
        let ai = pts.len();
        pts.push([x, 0.0, z]);
        arm_ids.push(ai as i64);
    }
    lines.push_cell(&arm_ids);
    // Lamp shade (cone/dome at arm end)
    let shade_z = height - arm_length * 0.3;
    let shade_x = arm_length;
    let sc = pts.len();
    pts.push([shade_x, 0.0, shade_z]);
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([
            shade_x + shade_radius * a.cos(),
            shade_radius * a.sin(),
            shade_z - shade_radius * 0.5,
        ]);
    }
    for i in 0..res {
        let j = if i + 1 < res { sc + 2 + i } else { sc + 1 };
        polys.push_cell(&[sc as i64, (sc + 1 + i) as i64, j as i64]);
    }
    // Base plate
    let bp = pts.len();
    let br = pole_radius * 3.0;
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
        let l = lamp_post(5.0, 0.1, 1.5, 0.3, 8);
        assert!(l.polys.num_cells() > 10);
    }
}
