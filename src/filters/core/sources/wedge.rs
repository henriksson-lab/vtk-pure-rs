//! Wedge (triangular prism) geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Create a wedge (triangular prism) with given base triangle and height.
pub fn wedge(base: [[f64; 3]; 3], height: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    // Bottom face
    for p in &base {
        pts.push(*p);
    }
    // Top face (offset along Z)
    for p in &base {
        pts.push([p[0], p[1], p[2] + height]);
    }

    let mut polys = CellArray::new();
    // Bottom face (reversed winding for outward normal)
    polys.push_cell(&[2, 1, 0]);
    // Top face
    polys.push_cell(&[3, 4, 5]);
    // Side faces
    polys.push_cell(&[0, 1, 4, 3]);
    polys.push_cell(&[1, 2, 5, 4]);
    polys.push_cell(&[2, 0, 3, 5]);

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Create a regular wedge centered at origin.
pub fn regular_wedge(size: f64, height: f64) -> PolyData {
    let s = size * 0.5;
    let h = s * 3.0f64.sqrt() / 2.0;
    wedge(
        [
            [-s, -h / 3.0, -height * 0.5],
            [s, -h / 3.0, -height * 0.5],
            [0.0, 2.0 * h / 3.0, -height * 0.5],
        ],
        height,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_wedge() {
        let w = wedge([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]], 2.0);
        assert_eq!(w.points.len(), 6);
        assert_eq!(w.polys.num_cells(), 5); // 2 caps + 3 sides
    }
    #[test]
    fn test_regular() {
        let w = regular_wedge(2.0, 3.0);
        assert_eq!(w.points.len(), 6);
    }
}
