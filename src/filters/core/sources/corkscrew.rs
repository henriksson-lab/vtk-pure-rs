//! Corkscrew (wine opener) with helix and handle.
use crate::data::{CellArray, Points, PolyData};

pub fn corkscrew(helix_length: f64, helix_radius: f64, turns: f64, handle_width: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Handle (T-shape)
    let h0 = pts.len();
    pts.push([-handle_width / 2.0, 0.0, helix_length + 0.5]);
    let h1 = pts.len();
    pts.push([handle_width / 2.0, 0.0, helix_length + 0.5]);
    lines.push_cell(&[h0 as i64, h1 as i64]);
    let h2 = pts.len();
    pts.push([0.0, 0.0, helix_length + 0.5]);
    let h3 = pts.len();
    pts.push([0.0, 0.0, helix_length]);
    lines.push_cell(&[h2 as i64, h3 as i64]);
    // Shaft
    let shaft_top = h3;
    let shaft_bot = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    lines.push_cell(&[shaft_top as i64, shaft_bot as i64]);
    // Helix (spiral worm)
    let n_pts = (turns * 20.0) as usize;
    let hb = pts.len();
    for i in 0..=n_pts {
        let t = i as f64 / n_pts as f64;
        let angle = 2.0 * std::f64::consts::PI * turns * t;
        let z = helix_length * t;
        pts.push([helix_radius * angle.cos(), helix_radius * angle.sin(), z]);
    }
    for i in 0..n_pts {
        lines.push_cell(&[(hb + i) as i64, (hb + i + 1) as i64]);
    }
    // Tip (pointed)
    let tip = pts.len();
    pts.push([helix_radius * 0.5, 0.0, -0.1]);
    lines.push_cell(&[hb as i64, tip as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_corkscrew() {
        let m = corkscrew(3.0, 0.3, 5.0, 2.0);
        assert!(m.points.len() > 80);
        assert!(m.lines.num_cells() > 80);
    }
}
