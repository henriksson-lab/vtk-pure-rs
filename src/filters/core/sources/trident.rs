//! Trident (three-pronged spear).
use crate::data::{CellArray, Points, PolyData};

pub fn trident(shaft_length: f64, prong_length: f64, spread: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Shaft
    let s0 = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let s1 = pts.len();
    pts.push([0.0, 0.0, shaft_length]);
    lines.push_cell(&[s0 as i64, s1 as i64]);
    // Center prong
    let c = pts.len();
    pts.push([0.0, 0.0, shaft_length + prong_length]);
    lines.push_cell(&[s1 as i64, c as i64]);
    // Left prong
    let l = pts.len();
    pts.push([-spread, 0.0, shaft_length + prong_length * 0.85]);
    let lm = pts.len();
    pts.push([-spread * 0.3, 0.0, shaft_length]);
    lines.push_cell(&[s1 as i64, lm as i64]);
    lines.push_cell(&[lm as i64, l as i64]);
    // Right prong
    let r = pts.len();
    pts.push([spread, 0.0, shaft_length + prong_length * 0.85]);
    let rm = pts.len();
    pts.push([spread * 0.3, 0.0, shaft_length]);
    lines.push_cell(&[s1 as i64, rm as i64]);
    lines.push_cell(&[rm as i64, r as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_trident() {
        let m = trident(5.0, 2.0, 0.8);
        assert!(m.points.len() >= 7);
        assert!(m.lines.num_cells() >= 5);
    }
}
