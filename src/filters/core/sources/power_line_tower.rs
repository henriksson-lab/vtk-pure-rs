//! Power line tower (pylon) with cross-arms.
use crate::data::{CellArray, Points, PolyData};
pub fn power_line_tower(
    height: f64,
    base_width: f64,
    top_width: f64,
    arm_length: f64,
    arm_height: f64,
) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let hw_b = base_width / 2.0;
    let hw_t = top_width / 2.0;
    // Four legs
    let b = pts.len();
    pts.push([-hw_b, -hw_b, 0.0]);
    pts.push([hw_b, -hw_b, 0.0]);
    pts.push([hw_b, hw_b, 0.0]);
    pts.push([-hw_b, hw_b, 0.0]);
    pts.push([-hw_t, -hw_t, height]);
    pts.push([hw_t, -hw_t, height]);
    pts.push([hw_t, hw_t, height]);
    pts.push([-hw_t, hw_t, height]);
    for i in 0..4 {
        lines.push_cell(&[(b + i) as i64, (b + 4 + i) as i64]);
    }
    // Horizontal bracing at 1/3 and 2/3
    for frac in [0.33, 0.67] {
        let z = height * frac;
        let w = hw_b + (hw_t - hw_b) * frac;
        let rb = pts.len();
        pts.push([-w, -w, z]);
        pts.push([w, -w, z]);
        pts.push([w, w, z]);
        pts.push([-w, w, z]);
        for i in 0..4 {
            lines.push_cell(&[(rb + i) as i64, (rb + (i + 1) % 4) as i64]);
        }
    }
    // Cross-arms
    let ah = height - arm_height;
    let ab = pts.len();
    pts.push([-arm_length, 0.0, ah]);
    pts.push([arm_length, 0.0, ah]);
    lines.push_cell(&[ab as i64, (ab + 1) as i64]);
    // Insulators (short verticals at arm ends)
    for &x in &[-arm_length, 0.0, arm_length] {
        let ib = pts.len();
        pts.push([x, 0.0, ah]);
        pts.push([x, 0.0, ah - arm_height * 0.3]);
        lines.push_cell(&[ib as i64, (ib + 1) as i64]);
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
        let t = power_line_tower(20.0, 4.0, 1.5, 6.0, 3.0);
        assert!(t.lines.num_cells() > 10);
    }
}
