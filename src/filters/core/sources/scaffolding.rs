//! Construction scaffolding geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn scaffolding(width: f64, height: f64, depth: f64, levels: usize) -> PolyData {
    let nl = levels.max(1);
    let level_h = height / nl as f64;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // Verticals at 4 corners
    for &x in &[0.0, width] {
        for &y in &[0.0, depth] {
            let b = pts.len();
            pts.push([x, y, 0.0]);
            pts.push([x, y, height]);
            lines.push_cell(&[b as i64, (b + 1) as i64]);
        }
    }
    // Horizontal frames at each level
    for li in 0..=nl {
        let z = li as f64 * level_h;
        let b = pts.len();
        pts.push([0.0, 0.0, z]);
        pts.push([width, 0.0, z]);
        pts.push([width, depth, z]);
        pts.push([0.0, depth, z]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
        lines.push_cell(&[(b + 1) as i64, (b + 2) as i64]);
        lines.push_cell(&[(b + 2) as i64, (b + 3) as i64]);
        lines.push_cell(&[(b + 3) as i64, b as i64]);
        // Platform at each level
        if li > 0 {
            polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64, (b + 3) as i64]);
        }
    }
    // Cross bracing on front
    for li in 0..nl {
        let z0 = li as f64 * level_h;
        let z1 = (li + 1) as f64 * level_h;
        let b = pts.len();
        pts.push([0.0, 0.0, z0]);
        pts.push([width, 0.0, z1]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
        let b2 = pts.len();
        pts.push([width, 0.0, z0]);
        pts.push([0.0, 0.0, z1]);
        lines.push_cell(&[b2 as i64, (b2 + 1) as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let s = scaffolding(3.0, 6.0, 1.5, 3);
        assert!(s.lines.num_cells() > 15);
        assert!(s.polys.num_cells() >= 3);
    }
}
