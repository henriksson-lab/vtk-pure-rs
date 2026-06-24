//! Tent geometry (poles + fabric surface).
use crate::data::{CellArray, Points, PolyData};
pub fn a_frame_tent(width: f64, length: f64, height: f64) -> PolyData {
    let hw = width / 2.0;
    let hl = length / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Base corners
    pts.push([-hw, -hl, 0.0]);
    pts.push([hw, -hl, 0.0]);
    pts.push([hw, hl, 0.0]);
    pts.push([-hw, hl, 0.0]);
    // Ridge
    pts.push([0.0, -hl, height]);
    pts.push([0.0, hl, height]);
    // Fabric panels
    polys.push_cell(&[0, 4, 5, 3]); // left slope
    polys.push_cell(&[1, 2, 5, 4]); // right slope
                                    // Front triangle
    polys.push_cell(&[0, 1, 4]);
    // Back triangle
    polys.push_cell(&[2, 3, 5]);
    // Floor
    polys.push_cell(&[0, 3, 2, 1]);
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn tipi(radius: f64, height: f64, num_poles: usize) -> PolyData {
    let np = num_poles.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    pts.push([0.0, 0.0, height]); // apex
    for i in 0..np {
        let a = 2.0 * std::f64::consts::PI * i as f64 / np as f64;
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
    }
    // Poles
    for i in 0..np {
        lines.push_cell(&[0, (i + 1) as i64]);
    }
    // Fabric (cone)
    for i in 0..np {
        let j = if i + 1 < np { i + 2 } else { 1 };
        polys.push_cell(&[0, (i + 1) as i64, j as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_a_frame() {
        let t = a_frame_tent(3.0, 5.0, 2.0);
        assert_eq!(t.polys.num_cells(), 5);
    }
    #[test]
    fn test_tipi() {
        let t = tipi(2.0, 3.0, 8);
        assert_eq!(t.polys.num_cells(), 8);
        assert_eq!(t.lines.num_cells(), 8);
    }
}
