//! Trebuchet (medieval siege weapon).
use crate::data::{CellArray, Points, PolyData};

pub fn trebuchet(height: f64, arm_length: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // A-frame supports
    let a0 = pts.len();
    pts.push([-1.0, -0.5, 0.0]);
    let a1 = pts.len();
    pts.push([-1.0, 0.5, 0.0]);
    let a2 = pts.len();
    pts.push([-1.0, 0.0, height]);
    lines.push_cell(&[a0 as i64, a2 as i64]);
    lines.push_cell(&[a1 as i64, a2 as i64]);
    let b0 = pts.len();
    pts.push([1.0, -0.5, 0.0]);
    let b1 = pts.len();
    pts.push([1.0, 0.5, 0.0]);
    let b2 = pts.len();
    pts.push([1.0, 0.0, height]);
    lines.push_cell(&[b0 as i64, b2 as i64]);
    lines.push_cell(&[b1 as i64, b2 as i64]);
    // Axle
    lines.push_cell(&[a2 as i64, b2 as i64]);
    // Beam (arm)
    let pivot = [0.0, 0.0, height];
    let short_end = pts.len();
    pts.push([pivot[0] - arm_length * 0.3, 0.0, height]);
    let long_end = pts.len();
    pts.push([pivot[0] + arm_length * 0.7, 0.0, height + 0.5]);
    lines.push_cell(&[short_end as i64, long_end as i64]);
    // Counterweight (hanging from short end)
    let cw = pts.len();
    pts.push([pivot[0] - arm_length * 0.3, 0.0, height - 2.0]);
    lines.push_cell(&[short_end as i64, cw as i64]);
    // Sling (from long end)
    let sl = pts.len();
    pts.push([pivot[0] + arm_length * 0.7 + 1.0, 0.0, height - 1.0]);
    lines.push_cell(&[long_end as i64, sl as i64]);
    // Base frame
    lines.push_cell(&[a0 as i64, b0 as i64]);
    lines.push_cell(&[a1 as i64, b1 as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_trebuchet() {
        let m = trebuchet(5.0, 8.0);
        assert!(m.points.len() > 8);
        assert!(m.lines.num_cells() > 6);
    }
}
