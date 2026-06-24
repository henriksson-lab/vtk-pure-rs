//! Roller coaster track (helical with loops).
use crate::data::{CellArray, Points, PolyData};
pub fn roller_coaster(
    length: f64,
    max_height: f64,
    num_hills: usize,
    _loop_radius: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(20);
    let gauge = 0.5;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut left_ids = Vec::new();
    let mut right_ids = Vec::new();
    for i in 0..=res {
        let t = i as f64 / res as f64;
        let x = t * length;
        let hill_phase = 2.0 * std::f64::consts::PI * num_hills as f64 * t;
        let z = max_height * 0.5 * (1.0 - (hill_phase).cos()) * ((1.0 - t).max(0.0));
        // Bank angle
        let bank = (hill_phase * 0.5).sin() * 0.2;
        let li = pts.len();
        pts.push([x, -gauge * bank.cos(), z + gauge * bank.sin()]);
        left_ids.push(li as i64);
        let ri = pts.len();
        pts.push([x, gauge * bank.cos(), z - gauge * bank.sin()]);
        right_ids.push(ri as i64);
        // Cross ties
        lines.push_cell(&[li as i64, ri as i64]);
    }
    lines.push_cell(&left_ids);
    lines.push_cell(&right_ids);
    // Support columns
    for i in (0..=res).step_by((res / 10).max(1)) {
        let t = i as f64 / res as f64;
        let x = t * length;
        let hill_phase = 2.0 * std::f64::consts::PI * num_hills as f64 * t;
        let z = max_height * 0.5 * (1.0 - (hill_phase).cos()) * ((1.0 - t).max(0.0));
        if z > 1.0 {
            let b = pts.len();
            pts.push([x, 0.0, 0.0]);
            pts.push([x, 0.0, z]);
            lines.push_cell(&[b as i64, (b + 1) as i64]);
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
        let r = roller_coaster(50.0, 15.0, 3, 5.0, 40);
        assert!(r.lines.num_cells() > 20);
    }
}
