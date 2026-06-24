//! Roller coaster with loop-the-loop.
use crate::data::{CellArray, Points, PolyData};
pub fn roller_coaster_with_loop(
    approach_l: f64,
    loop_r: f64,
    exit_l: f64,
    gauge: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(20);
    let hg = gauge / 2.0;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Approach (straight, rising)
    let approach_steps = res / 4;
    let loop_bottom_z = loop_r * 2.0 + 1.0;
    let mut left = Vec::new();
    let mut right = Vec::new();
    for i in 0..=approach_steps {
        let t = i as f64 / approach_steps as f64;
        let x = -approach_l * (1.0 - t);
        let z = loop_bottom_z * t;
        let li = pts.len();
        pts.push([x, -hg, z]);
        left.push(li as i64);
        let ri = pts.len();
        pts.push([x, hg, z]);
        right.push(ri as i64);
        lines.push_cell(&[li as i64, ri as i64]);
    }
    // Loop
    let loop_steps = res;
    for i in 1..=loop_steps {
        let a =
            2.0 * std::f64::consts::PI * i as f64 / loop_steps as f64 - std::f64::consts::FRAC_PI_2;
        let x = loop_r * a.cos();
        let z = loop_bottom_z + loop_r + loop_r * a.sin();
        let li = pts.len();
        pts.push([x, -hg, z]);
        left.push(li as i64);
        let ri = pts.len();
        pts.push([x, hg, z]);
        right.push(ri as i64);
        lines.push_cell(&[li as i64, ri as i64]);
    }
    // Exit (descending)
    let exit_steps = res / 4;
    for i in 1..=exit_steps {
        let t = i as f64 / exit_steps as f64;
        let x = exit_l * t;
        let z = loop_bottom_z * (1.0 - t);
        let li = pts.len();
        pts.push([x, -hg, z]);
        left.push(li as i64);
        let ri = pts.len();
        pts.push([x, hg, z]);
        right.push(ri as i64);
        lines.push_cell(&[li as i64, ri as i64]);
    }
    lines.push_cell(&left);
    lines.push_cell(&right);
    // Support columns
    for i in (0..left.len()).step_by((left.len() / 8).max(1)) {
        let p = pts.get(left[i as usize] as usize);
        if p[2] > 1.0 {
            let sb = pts.len();
            pts.push([p[0], p[1], 0.0]);
            pts.push([p[0], p[1], p[2]]);
            lines.push_cell(&[sb as i64, (sb + 1) as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let r = roller_coaster_with_loop(15.0, 5.0, 15.0, 1.0, 32);
        assert!(r.lines.num_cells() > 20);
    }
}
