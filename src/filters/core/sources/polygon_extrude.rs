//! Extrude a 2D polygon into a 3D solid.

use crate::data::{CellArray, Points, PolyData};

/// Extrude a polygon defined by 2D points along the Z axis.
///
/// Creates a closed solid with top face, bottom face, and side quads.
pub fn extrude_polygon(outline: &[[f64; 2]], height: f64) -> PolyData {
    let n = outline.len();
    if n < 3 {
        return PolyData::new();
    }

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Bottom vertices
    for p in outline {
        points.push([p[0], p[1], 0.0]);
    }
    // Top vertices
    for p in outline {
        points.push([p[0], p[1], height]);
    }

    // Bottom face (fan triangulation, reversed for outward normal)
    for i in 1..n - 1 {
        polys.push_cell(&[0, (i + 1) as i64, i as i64]);
    }
    // Top face
    for i in 1..n - 1 {
        polys.push_cell(&[n as i64, (n + i) as i64, (n + i + 1) as i64]);
    }
    // Side quads
    for i in 0..n {
        let i0 = i as i64;
        let i1 = ((i + 1) % n) as i64;
        let i2 = i1 + n as i64;
        let i3 = i0 + n as i64;
        polys.push_cell(&[i0, i1, i2]);
        polys.push_cell(&[i0, i2, i3]);
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

/// Extrude a polygon along a direction vector.
pub fn extrude_polygon_along(outline: &[[f64; 2]], direction: [f64; 3], distance: f64) -> PolyData {
    let n = outline.len();
    if n < 3 {
        return PolyData::new();
    }
    let len = (direction[0].powi(2) + direction[1].powi(2) + direction[2].powi(2)).sqrt();
    if len < 1e-15 {
        return PolyData::new();
    }
    let d = [
        direction[0] / len * distance,
        direction[1] / len * distance,
        direction[2] / len * distance,
    ];

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    for p in outline {
        points.push([p[0], p[1], 0.0]);
    }
    for p in outline {
        points.push([p[0] + d[0], p[1] + d[1], d[2]]);
    }

    for i in 1..n - 1 {
        polys.push_cell(&[0, (i + 1) as i64, i as i64]);
    }
    for i in 1..n - 1 {
        polys.push_cell(&[n as i64, (n + i) as i64, (n + i + 1) as i64]);
    }
    for i in 0..n {
        let i0 = i as i64;
        let i1 = ((i + 1) % n) as i64;
        polys.push_cell(&[i0, i1, i1 + n as i64]);
        polys.push_cell(&[i0, i1 + n as i64, i0 + n as i64]);
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_extrude() {
        let outline = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let solid = extrude_polygon(&outline, 2.0);
        assert_eq!(solid.points.len(), 8);
        assert!(solid.polys.num_cells() > 8); // 2 cap fans + 4 side quads
    }

    #[test]
    fn triangle_extrude() {
        let outline = [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let solid = extrude_polygon(&outline, 1.0);
        assert_eq!(solid.points.len(), 6);
    }

    #[test]
    fn along_direction() {
        let outline = [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let solid = extrude_polygon_along(&outline, [1.0, 0.0, 1.0], 2.0);
        assert_eq!(solid.points.len(), 6);
    }
}
