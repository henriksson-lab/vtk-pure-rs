//! Violin body outline (wireframe profile).
use crate::data::{CellArray, Points, PolyData};

pub fn violin(length: f64, n_pts: usize) -> PolyData {
    let n = n_pts.max(20);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Parametric violin profile (one side)
    let mut right_pts = Vec::new();
    let mut left_pts = Vec::new();
    for i in 0..=n {
        let t = i as f64 / n as f64;
        let z = t * length;
        // Width varies: wide at bottom, narrow at waist, wide at top
        let w = length
            * 0.2
            * (1.0 + 0.3 * (2.0 * std::f64::consts::PI * t).cos()
                - 0.15 * (4.0 * std::f64::consts::PI * t).cos());
        right_pts.push(pts.len());
        pts.push([w, 0.0, z]);
        left_pts.push(pts.len());
        pts.push([-w, 0.0, z]);
    }
    for i in 0..n {
        lines.push_cell(&[right_pts[i] as i64, right_pts[i + 1] as i64]);
        lines.push_cell(&[left_pts[i] as i64, left_pts[i + 1] as i64]);
    }
    // Connect top and bottom
    lines.push_cell(&[right_pts[0] as i64, left_pts[0] as i64]);
    lines.push_cell(&[right_pts[n] as i64, left_pts[n] as i64]);
    // F-holes (simplified as short line segments)
    let fh_z = length * 0.45;
    let fh_w = length * 0.06;
    let f0 = pts.len();
    pts.push([fh_w, 0.0, fh_z - length * 0.05]);
    let f1 = pts.len();
    pts.push([fh_w, 0.0, fh_z + length * 0.05]);
    lines.push_cell(&[f0 as i64, f1 as i64]);
    let f2 = pts.len();
    pts.push([-fh_w, 0.0, fh_z - length * 0.05]);
    let f3 = pts.len();
    pts.push([-fh_w, 0.0, fh_z + length * 0.05]);
    lines.push_cell(&[f2 as i64, f3 as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_violin() {
        let m = violin(35.0, 30);
        assert!(m.points.len() > 40);
        assert!(m.lines.num_cells() > 30);
    }
}
