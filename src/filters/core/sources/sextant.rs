//! Navigation sextant (arc with mirrors and telescope).
use crate::data::{CellArray, Points, PolyData};

pub fn sextant(radius: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let na = 20;
    // Arc (60 degrees = 1/6 of circle, hence "sextant")
    let arc_base = pts.len();
    for j in 0..=na {
        let angle = -std::f64::consts::PI / 6.0 + std::f64::consts::PI / 3.0 * j as f64 / na as f64;
        pts.push([radius * angle.cos(), 0.0, radius * angle.sin()]);
    }
    for j in 0..na {
        lines.push_cell(&[(arc_base + j) as i64, (arc_base + j + 1) as i64]);
    }
    // Frame arms from center to arc endpoints
    let center = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    lines.push_cell(&[center as i64, arc_base as i64]);
    lines.push_cell(&[center as i64, (arc_base + na) as i64]);
    // Index arm (movable arm)
    let idx_tip = pts.len();
    let mid_angle = std::f64::consts::PI / 6.0;
    pts.push([
        radius * 0.95 * mid_angle.cos(),
        0.0,
        radius * 0.95 * mid_angle.sin(),
    ]);
    lines.push_cell(&[center as i64, idx_tip as i64]);
    // Telescope (short line from center)
    let tel = pts.len();
    pts.push([-radius * 0.4, 0.0, radius * 0.3]);
    lines.push_cell(&[center as i64, tel as i64]);
    // Handle
    let handle = pts.len();
    pts.push([0.0, 0.0, -radius * 0.3]);
    lines.push_cell(&[center as i64, handle as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sextant() {
        let m = sextant(5.0);
        assert!(m.points.len() > 20);
        assert!(m.lines.num_cells() > 10);
    }
}
