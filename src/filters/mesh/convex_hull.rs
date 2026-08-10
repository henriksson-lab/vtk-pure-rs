//! 2D and 3D convex hull computation.

use crate::data::{CellArray, PolyData};

pub use crate::filters::mesh::convex_hull_2d::convex_hull_2d;

/// Extract convex hull as wireframe (lines).
pub fn convex_hull_2d_wireframe(mesh: &PolyData) -> PolyData {
    let hull = convex_hull_2d(mesh);
    if hull.polys.num_cells() == 0 {
        return hull;
    }
    let cell: Vec<i64> = hull.polys.iter().next().unwrap().to_vec();
    let n = cell.len();
    let mut lines = CellArray::new();
    for i in 0..n {
        lines.push_cell(&[cell[i], cell[(i + 1) % n]]);
    }
    let mut result = PolyData::new();
    result.points = hull.points;
    result.lines = lines;
    result
}

/// Compute convex hull area (2D, XY plane).
pub fn convex_hull_area(mesh: &PolyData) -> f64 {
    let hull = convex_hull_2d(mesh);
    if hull.polys.num_cells() == 0 {
        return 0.0;
    }
    let cell: Vec<i64> = hull.polys.iter().next().unwrap().to_vec();
    let n = cell.len();
    let mut area = 0.0;
    for i in 0..n {
        let a = hull.points.get(cell[i] as usize);
        let b = hull.points.get(cell[(i + 1) % n] as usize);
        area += a[0] * b[1] - b[0] * a[1];
    }
    area.abs() * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_square() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.points.push([0.5, 0.5, 0.0]); // interior point
        let hull = convex_hull_2d(&mesh);
        let cell: Vec<i64> = hull.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell.len(), 4); // interior point excluded
    }
    #[test]
    fn test_area() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([2.0, 0.0, 0.0]);
        mesh.points.push([2.0, 3.0, 0.0]);
        mesh.points.push([0.0, 3.0, 0.0]);
        let a = convex_hull_area(&mesh);
        assert!((a - 6.0).abs() < 1e-10);
    }
}
