//! Pyramid geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Generate a pyramid with a square base and triangular sides.
pub fn pyramid(base_size: f64, height: f64) -> PolyData {
    let s = base_size / 2.0;
    let pts = vec![
        [-s, -s, 0.0],
        [s, -s, 0.0],
        [s, s, 0.0],
        [-s, s, 0.0],       // base
        [0.0, 0.0, height], // apex
    ];
    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }

    let mut polys = CellArray::new();
    // Base (two triangles)
    polys.push_cell(&[0, 2, 1]);
    polys.push_cell(&[0, 3, 2]);
    // Sides
    polys.push_cell(&[0, 1, 4]);
    polys.push_cell(&[1, 2, 4]);
    polys.push_cell(&[2, 3, 4]);
    polys.push_cell(&[3, 0, 4]);

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

/// Generate a truncated pyramid (frustum with rectangular cross-section).
pub fn truncated_pyramid(bottom_size: f64, top_size: f64, height: f64) -> PolyData {
    let sb = bottom_size / 2.0;
    let st = top_size / 2.0;
    let pts = vec![
        [-sb, -sb, 0.0],
        [sb, -sb, 0.0],
        [sb, sb, 0.0],
        [-sb, sb, 0.0],
        [-st, -st, height],
        [st, -st, height],
        [st, st, height],
        [-st, st, height],
    ];
    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }
    let mut polys = CellArray::new();
    // Bottom
    polys.push_cell(&[0, 2, 1]);
    polys.push_cell(&[0, 3, 2]);
    // Top
    polys.push_cell(&[4, 5, 6]);
    polys.push_cell(&[4, 6, 7]);
    // Sides
    polys.push_cell(&[0, 1, 5]);
    polys.push_cell(&[0, 5, 4]);
    polys.push_cell(&[1, 2, 6]);
    polys.push_cell(&[1, 6, 5]);
    polys.push_cell(&[2, 3, 7]);
    polys.push_cell(&[2, 7, 6]);
    polys.push_cell(&[3, 0, 4]);
    polys.push_cell(&[3, 4, 7]);

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_pyramid() {
        let p = pyramid(2.0, 3.0);
        assert_eq!(p.points.len(), 5);
        assert_eq!(p.polys.num_cells(), 6);
        assert!((p.points.get(4)[2] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn truncated() {
        let p = truncated_pyramid(2.0, 1.0, 3.0);
        assert_eq!(p.points.len(), 8);
        assert_eq!(p.polys.num_cells(), 12);
    }
}
