//! Mirror mesh across an arbitrary plane.
use crate::data::{CellArray, Points, PolyData};

pub fn mirror_plane(mesh: &PolyData, point: [f64; 3], normal: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let nl = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    let nn = if nl > 1e-15 {
        [normal[0] / nl, normal[1] / nl, normal[2] / nl]
    } else {
        [0.0, 0.0, 1.0]
    };
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = (p[0] - point[0]) * nn[0] + (p[1] - point[1]) * nn[1] + (p[2] - point[2]) * nn[2];
        pts.push([
            p[0] - 2.0 * d * nn[0],
            p[1] - 2.0 * d * nn[1],
            p[2] - 2.0 * d * nn[2],
        ]);
    }
    // Reverse winding for mirrored faces
    let mut polys = CellArray::new();
    for cell in mesh.polys.iter() {
        let rev: Vec<i64> = cell.iter().rev().copied().collect();
        polys.push_cell(&rev);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result.lines = mesh.lines.clone();
    result.verts = mesh.verts.clone();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mirror() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = mirror_plane(&mesh, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let p = r.points.get(0);
        assert!((p[0] - (-1.0)).abs() < 1e-9); // reflected across YZ plane
    }
}
