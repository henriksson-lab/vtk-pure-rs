//! Rope bridge with catenary cables and plank deck.
use crate::data::{CellArray, Points, PolyData};

pub fn rope_bridge(span: f64, sag: f64, width: f64, n_planks: usize) -> PolyData {
    let np = n_planks.max(5);
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Main cables (parabolic catenary approximation on each side)
    for &side in &[-1.0f64, 1.0] {
        let y = side * hw;
        let cable_base = pts.len();
        for i in 0..=np {
            let t = i as f64 / np as f64;
            let x = span * (t - 0.5);
            let z = sag * (2.0 * t - 1.0).powi(2);
            pts.push([x, y, z]);
        }
        for i in 0..np {
            lines.push_cell(&[(cable_base + i) as i64, (cable_base + i + 1) as i64]);
        }
    }
    // Planks connecting left and right at each station
    for i in 0..=np {
        let t = i as f64 / np as f64;
        let x = span * (t - 0.5);
        let z = sag * (2.0 * t - 1.0).powi(2) - 0.1;
        let pb = pts.len();
        let pw = width * 0.05; // plank thickness
        pts.push([x - pw, -hw, z]);
        pts.push([x + pw, -hw, z]);
        pts.push([x + pw, hw, z]);
        pts.push([x - pw, hw, z]);
        polys.push_cell(&[pb as i64, (pb + 1) as i64, (pb + 2) as i64, (pb + 3) as i64]);
        // Vertical ropes from cable to plank
        let left_cable = i;
        let right_cable = (np + 1) + i;
        lines.push_cell(&[left_cable as i64, pb as i64]);
        lines.push_cell(&[right_cable as i64, (pb + 3) as i64]);
    }
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
    fn test_rope_bridge() {
        let m = rope_bridge(20.0, 3.0, 2.0, 10);
        assert!(m.points.len() > 40);
        assert!(m.polys.num_cells() >= 10);
        assert!(m.lines.num_cells() > 20);
    }
}
