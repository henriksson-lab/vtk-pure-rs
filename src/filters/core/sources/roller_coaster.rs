//! Roller coaster track with a vertical loop.
use crate::data::{CellArray, Points, PolyData};

pub fn roller_coaster(loop_radius: f64, track_width: f64, n_pts: usize) -> PolyData {
    let n = n_pts.max(40);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let hw = track_width / 2.0;
    // Track path: approach, loop, exit
    let total_len = 4.0 * loop_radius + 2.0 * std::f64::consts::PI * loop_radius;
    let mut left_rail = Vec::new();
    let mut right_rail = Vec::new();
    for i in 0..=n {
        let t = i as f64 / n as f64;
        let s = t * total_len;
        let (x, z, _nx, _nz);
        if s < 2.0 * loop_radius {
            // Approach ramp
            x = -2.0 * loop_radius + s;
            z = s * 0.5; // gentle slope up
            _nx = 0.0;
            _nz = 1.0;
        } else if s < 2.0 * loop_radius + 2.0 * std::f64::consts::PI * loop_radius {
            // Vertical loop
            let angle = (s - 2.0 * loop_radius) / loop_radius;
            x = loop_radius * angle.sin();
            z = loop_radius + loop_radius * (1.0 - angle.cos());
            _nx = -angle.sin();
            _nz = angle.cos();
        } else {
            // Exit
            let ds = s - 2.0 * loop_radius - 2.0 * std::f64::consts::PI * loop_radius;
            x = ds;
            z = 2.0 * loop_radius - ds * 0.5; // slope down
            _nx = 0.0;
            _nz = 1.0;
        }
        let l = pts.len();
        pts.push([x, -hw, z]);
        left_rail.push(l);
        let r = pts.len();
        pts.push([x, hw, z]);
        right_rail.push(r);
    }
    for i in 0..n {
        lines.push_cell(&[left_rail[i] as i64, left_rail[i + 1] as i64]);
        lines.push_cell(&[right_rail[i] as i64, right_rail[i + 1] as i64]);
    }
    // Cross ties every few segments
    let tie_step = n / 20;
    let tie_step = tie_step.max(1);
    for i in (0..=n).step_by(tie_step) {
        lines.push_cell(&[left_rail[i] as i64, right_rail[i] as i64]);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_coaster() {
        let m = roller_coaster(5.0, 1.0, 60);
        assert!(m.points.len() > 100);
        assert!(m.lines.num_cells() > 50);
    }
}
