//! Lattice tower (antenna/transmission tower) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn lattice_tower(base_width: f64, top_width: f64, height: f64, sections: usize) -> PolyData {
    let ns = sections.max(1);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    for i in 0..=ns {
        let t = i as f64 / ns as f64;
        let z = t * height;
        let w = base_width * (1.0 - t) + top_width * t;
        let hw = w / 2.0;
        pts.push([-hw, -hw, z]);
        pts.push([hw, -hw, z]);
        pts.push([hw, hw, z]);
        pts.push([-hw, hw, z]);
    }
    for i in 0..ns {
        let b = i * 4;
        let t = (i + 1) * 4;
        // Horizontal edges
        for j in 0..4 {
            let j1 = (j + 1) % 4;
            lines.push_cell(&[(b + j) as i64, (b + j1) as i64]);
        }
        // Vertical edges
        for j in 0..4 {
            lines.push_cell(&[(b + j) as i64, (t + j) as i64]);
        }
        // Cross bracing
        for j in 0..4 {
            lines.push_cell(&[(b + j) as i64, (t + (j + 1) % 4) as i64]);
        }
    }
    // Top ring
    let top = ns * 4;
    for j in 0..4 {
        let j1 = (j + 1) % 4;
        lines.push_cell(&[(top + j) as i64, (top + j1) as i64]);
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
        let t = lattice_tower(4.0, 1.0, 20.0, 5);
        assert!(t.lines.num_cells() > 30);
    }
}
