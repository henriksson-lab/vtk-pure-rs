//! Playground equipment geometry (swing set, slide).
use crate::data::{CellArray, Points, PolyData};
pub fn swing_set(width: f64, height: f64, num_swings: usize) -> PolyData {
    let ns = num_swings.max(1);
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // A-frame legs
    let leg_spread = 0.3;
    for side in [-hw, hw] {
        let b = pts.len();
        pts.push([side - leg_spread, 0.0, 0.0]);
        pts.push([side, 0.0, height]);
        pts.push([side + leg_spread, 0.0, 0.0]);
        pts.push([side, 0.0, height]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
        lines.push_cell(&[(b + 2) as i64, (b + 3) as i64]);
    }
    // Top bar
    let b = pts.len();
    pts.push([-hw, 0.0, height]);
    pts.push([hw, 0.0, height]);
    lines.push_cell(&[b as i64, (b + 1) as i64]);
    // Swings
    let spacing = width / (ns + 1) as f64;
    for i in 0..ns {
        let x = -hw + spacing * (i + 1) as f64;
        let b = pts.len();
        pts.push([x, 0.0, height]);
        pts.push([x - 0.1, 0.0, height * 0.3]);
        pts.push([x, 0.0, height]);
        pts.push([x + 0.1, 0.0, height * 0.3]);
        pts.push([x - 0.1, 0.0, height * 0.3]);
        pts.push([x + 0.1, 0.0, height * 0.3]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
        lines.push_cell(&[(b + 2) as i64, (b + 3) as i64]);
        lines.push_cell(&[(b + 4) as i64, (b + 5) as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
pub fn slide(height: f64, length: f64, width: f64, resolution: usize) -> PolyData {
    let res = resolution.max(4);
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..=res {
        let t = i as f64 / res as f64;
        let x = t * length;
        let z = height * (1.0 - t * t); // parabolic slide
        pts.push([x, -hw, z]);
        pts.push([x, hw, z]);
    }
    for i in 0..res {
        let b = i * 2;
        polys.push_cell(&[b as i64, (b + 1) as i64, (b + 3) as i64, (b + 2) as i64]);
    }
    // Ladder
    let _lb = pts.len();
    pts.push([0.0, -hw * 0.3, 0.0]);
    pts.push([0.0, -hw * 0.3, height]);
    pts.push([0.0, hw * 0.3, 0.0]);
    pts.push([0.0, hw * 0.3, height]);
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_swing() {
        let s = swing_set(4.0, 3.0, 3);
        assert!(s.lines.num_cells() > 5);
    }
    #[test]
    fn test_slide() {
        let s = slide(2.0, 4.0, 1.0, 8);
        assert!(s.polys.num_cells() >= 4);
    }
}
