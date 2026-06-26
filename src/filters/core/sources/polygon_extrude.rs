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
    let mut strips = CellArray::new();

    // Bottom vertices
    for p in outline {
        points.push([p[0], p[1], 0.0]);
    }
    // Top vertices
    for p in outline {
        points.push([p[0], p[1], height]);
    }

    let bottom: Vec<i64> = (0..n).map(|i| i as i64).collect();
    let top: Vec<i64> = (0..n).map(|i| (i + n) as i64).collect();
    polys.push_cell(&bottom);
    polys.push_cell(&top);

    // Side strips, matching vtkLinearExtrusionFilter boundary-edge output.
    for i in 0..n {
        let p1 = i as i64;
        let p2 = ((i + 1) % n) as i64;
        strips.push_cell(&[p1, p2, p1 + n as i64, p2 + n as i64]);
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh.strips = strips;
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
    let mut strips = CellArray::new();

    for p in outline {
        points.push([p[0], p[1], 0.0]);
    }
    for p in outline {
        points.push([p[0] + d[0], p[1] + d[1], d[2]]);
    }

    let bottom: Vec<i64> = (0..n).map(|i| i as i64).collect();
    let top: Vec<i64> = (0..n).map(|i| (i + n) as i64).collect();
    polys.push_cell(&bottom);
    polys.push_cell(&top);

    for i in 0..n {
        let p1 = i as i64;
        let p2 = ((i + 1) % n) as i64;
        strips.push_cell(&[p1, p2, p1 + n as i64, p2 + n as i64]);
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh.strips = strips;
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
        assert_eq!(solid.polys.num_cells(), 2);
        assert_eq!(solid.strips.num_cells(), 4);
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
