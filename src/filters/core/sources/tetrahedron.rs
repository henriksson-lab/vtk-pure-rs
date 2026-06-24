//! Tetrahedron geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Create a regular tetrahedron with given edge length, centered at origin.
pub fn regular_tetrahedron(edge_length: f64) -> PolyData {
    let a = edge_length;
    let h = a * (2.0 / 3.0f64).sqrt();
    let r = a / 3.0f64.sqrt();

    let verts = [
        [r, 0.0, -h / 4.0],
        [-r / 2.0, a / 2.0, -h / 4.0],
        [-r / 2.0, -a / 2.0, -h / 4.0],
        [0.0, 0.0, 3.0 * h / 4.0],
    ];

    let faces = [[0, 2, 1], [0, 1, 3], [1, 2, 3], [2, 0, 3]];

    let mut pts = Points::<f64>::new();
    for v in &verts {
        pts.push(*v);
    }
    let mut polys = CellArray::new();
    for f in &faces {
        polys.push_cell(&[f[0] as i64, f[1] as i64, f[2] as i64]);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Create a tetrahedron from 4 arbitrary points.
pub fn tetrahedron_from_points(p0: [f64; 3], p1: [f64; 3], p2: [f64; 3], p3: [f64; 3]) -> PolyData {
    let mut pts = Points::<f64>::new();
    pts.push(p0);
    pts.push(p1);
    pts.push(p2);
    pts.push(p3);
    let mut polys = CellArray::new();
    polys.push_cell(&[0, 2, 1]);
    polys.push_cell(&[0, 1, 3]);
    polys.push_cell(&[1, 2, 3]);
    polys.push_cell(&[2, 0, 3]);
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_regular() {
        let t = regular_tetrahedron(2.0);
        assert_eq!(t.points.len(), 4);
        assert_eq!(t.polys.num_cells(), 4);
    }
    #[test]
    fn test_from_points() {
        let t = tetrahedron_from_points(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        );
        assert_eq!(t.points.len(), 4);
        assert_eq!(t.polys.num_cells(), 4);
    }
}
