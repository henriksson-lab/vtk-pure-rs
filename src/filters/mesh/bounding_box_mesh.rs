//! Generate bounding box mesh from a PolyData.

use crate::data::{CellArray, Points, PolyData};

/// Create a wireframe bounding box around a mesh.
pub fn bounding_box_wireframe(mesh: &PolyData) -> PolyData {
    let (mn, mx) = bounds(mesh);
    let verts = [
        [mn[0], mn[1], mn[2]],
        [mx[0], mn[1], mn[2]],
        [mx[0], mx[1], mn[2]],
        [mn[0], mx[1], mn[2]],
        [mn[0], mn[1], mx[2]],
        [mx[0], mn[1], mx[2]],
        [mx[0], mx[1], mx[2]],
        [mn[0], mx[1], mx[2]],
    ];
    let mut pts = Points::<f64>::new();
    for v in &verts {
        pts.push(*v);
    }
    let mut lines = CellArray::new();
    let edges = [
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0],
        [4, 5],
        [5, 6],
        [6, 7],
        [7, 4],
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7],
    ];
    for e in &edges {
        lines.push_cell(&[e[0] as i64, e[1] as i64]);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
}

/// Create a solid bounding box (6 quad faces) around a mesh.
pub fn bounding_box_solid(mesh: &PolyData) -> PolyData {
    let (mn, mx) = bounds(mesh);
    let verts = [
        [mn[0], mn[1], mn[2]],
        [mx[0], mn[1], mn[2]],
        [mx[0], mx[1], mn[2]],
        [mn[0], mx[1], mn[2]],
        [mn[0], mn[1], mx[2]],
        [mx[0], mn[1], mx[2]],
        [mx[0], mx[1], mx[2]],
        [mn[0], mx[1], mx[2]],
    ];
    let mut pts = Points::<f64>::new();
    for v in &verts {
        pts.push(*v);
    }
    let mut polys = CellArray::new();
    let faces = [
        [0, 3, 2, 1],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [2, 3, 7, 6],
        [0, 4, 7, 3],
        [1, 2, 6, 5],
    ];
    for f in &faces {
        polys.push_cell(&[f[0] as i64, f[1] as i64, f[2] as i64, f[3] as i64]);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Get min/max bounds of mesh.
pub fn mesh_bounds(mesh: &PolyData) -> ([f64; 3], [f64; 3]) {
    bounds(mesh)
}

/// Get diagonal length of bounding box.
pub fn bounding_box_diagonal(mesh: &PolyData) -> f64 {
    let (mn, mx) = bounds(mesh);
    ((mx[0] - mn[0]).powi(2) + (mx[1] - mn[1]).powi(2) + (mx[2] - mn[2]).powi(2)).sqrt()
}

fn bounds(mesh: &PolyData) -> ([f64; 3], [f64; 3]) {
    if mesh.points.len() == 0 {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = [f64::INFINITY; 3];
    let mut mx = [f64::NEG_INFINITY; 3];
    for i in 0..mesh.points.len() {
        let p = mesh.points.get(i);
        for j in 0..3 {
            mn[j] = mn[j].min(p[j]);
            mx[j] = mx[j].max(p[j]);
        }
    }
    (mn, mx)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_wireframe() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 3.0, 4.0]],
            vec![[0, 1, 2]],
        );
        let bb = bounding_box_wireframe(&mesh);
        assert_eq!(bb.points.len(), 8);
        assert_eq!(bb.lines.num_cells(), 12);
    }
    #[test]
    fn test_solid() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let bb = bounding_box_solid(&mesh);
        assert_eq!(bb.polys.num_cells(), 6);
    }
    #[test]
    fn test_diagonal() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 4.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let d = bounding_box_diagonal(&mesh);
        assert!((d - 5.0).abs() < 1e-10);
    }
}
