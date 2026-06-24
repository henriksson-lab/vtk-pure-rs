//! Mirror mesh across planes.

use crate::data::{CellArray, PolyData};

/// Mirror mesh across the YZ plane (flip X).
pub fn mirror_x(mesh: &PolyData) -> PolyData {
    mirror_axis(mesh, 0)
}

/// Mirror mesh across the XZ plane (flip Y).
pub fn mirror_y(mesh: &PolyData) -> PolyData {
    mirror_axis(mesh, 1)
}

/// Mirror mesh across the XY plane (flip Z).
pub fn mirror_z(mesh: &PolyData) -> PolyData {
    mirror_axis(mesh, 2)
}

/// Mirror mesh across an arbitrary plane defined by point and normal.
pub fn mirror_plane(mesh: &PolyData, point: [f64; 3], normal: [f64; 3]) -> PolyData {
    let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if len < 1e-15 {
        return mesh.clone();
    }
    let n = [normal[0] / len, normal[1] / len, normal[2] / len];
    let mut result = mesh.clone();
    for i in 0..result.points.len() {
        let p = result.points.get(i);
        let d = (p[0] - point[0]) * n[0] + (p[1] - point[1]) * n[1] + (p[2] - point[2]) * n[2];
        result.points.set(
            i,
            [
                p[0] - 2.0 * d * n[0],
                p[1] - 2.0 * d * n[1],
                p[2] - 2.0 * d * n[2],
            ],
        );
    }
    // Reverse winding to maintain outward normals
    let mut new_polys = CellArray::new();
    for cell in result.polys.iter() {
        let mut reversed: Vec<i64> = cell.to_vec();
        reversed.reverse();
        new_polys.push_cell(&reversed);
    }
    result.polys = new_polys;
    result
}

fn mirror_axis(mesh: &PolyData, axis: usize) -> PolyData {
    let mut result = mesh.clone();
    for i in 0..result.points.len() {
        let mut p = result.points.get(i);
        p[axis] = -p[axis];
        result.points.set(i, p);
    }
    let mut new_polys = CellArray::new();
    for cell in result.polys.iter() {
        let mut reversed: Vec<i64> = cell.to_vec();
        reversed.reverse();
        new_polys.push_cell(&reversed);
    }
    result.polys = new_polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mirror_x() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = mirror_x(&mesh);
        let p = r.points.get(0);
        assert!((p[0] + 1.0).abs() < 1e-10);
    }
    #[test]
    fn test_mirror_plane() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [2.0, 2.0, 3.0], [1.5, 3.0, 3.0]],
            vec![[0, 1, 2]],
        );
        let r = mirror_plane(&mesh, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let p = r.points.get(0);
        assert!((p[0] + 1.0).abs() < 1e-10);
        assert!((p[1] - 2.0).abs() < 1e-10);
    }
    #[test]
    fn test_winding_reversed() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = mirror_z(&mesh);
        let cell: Vec<i64> = r.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell, vec![2, 1, 0]); // reversed
    }
}
