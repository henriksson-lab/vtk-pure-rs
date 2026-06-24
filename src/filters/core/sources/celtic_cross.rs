//! Celtic cross (cross with ring).
use crate::data::{CellArray, Points, PolyData};

pub fn celtic_cross(height: f64, arm_width: f64, ring_radius: f64, na: usize) -> PolyData {
    let na = na.max(16);
    let hw = arm_width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Vertical bar
    let cross_center_z = height * 0.6;
    let vb = pts.len();
    pts.push([-hw, 0.0, 0.0]);
    pts.push([hw, 0.0, 0.0]);
    pts.push([hw, 0.0, height]);
    pts.push([-hw, 0.0, height]);
    polys.push_cell(&[vb as i64, (vb + 1) as i64, (vb + 2) as i64, (vb + 3) as i64]);
    // Horizontal bar
    let arm_span = ring_radius * 1.5;
    let hb = pts.len();
    pts.push([-arm_span, 0.0, cross_center_z - hw]);
    pts.push([arm_span, 0.0, cross_center_z - hw]);
    pts.push([arm_span, 0.0, cross_center_z + hw]);
    pts.push([-arm_span, 0.0, cross_center_z + hw]);
    polys.push_cell(&[hb as i64, (hb + 1) as i64, (hb + 2) as i64, (hb + 3) as i64]);
    // Ring (circle at intersection)
    let rb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([
            ring_radius * a.cos(),
            0.01,
            cross_center_z + ring_radius * a.sin(),
        ]);
    }
    for j in 0..na {
        lines.push_cell(&[(rb + j) as i64, (rb + (j + 1) % na) as i64]);
    }
    // Base (trapezoidal)
    let bb = pts.len();
    pts.push([-hw * 2.0, 0.0, 0.0]);
    pts.push([hw * 2.0, 0.0, 0.0]);
    pts.push([hw * 1.5, 0.0, -height * 0.05]);
    pts.push([-hw * 1.5, 0.0, -height * 0.05]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 2) as i64, (bb + 3) as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_celtic() {
        let m = celtic_cross(5.0, 0.4, 1.0, 20);
        assert!(m.points.len() > 20);
        assert!(m.polys.num_cells() >= 3);
        assert!(m.lines.num_cells() >= 20);
    }
}
