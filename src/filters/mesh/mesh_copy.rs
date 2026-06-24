//! Mesh copy, merge, and append operations.

use crate::data::{CellArray, Points, PolyData};

/// Deep copy a mesh (no shared references).
pub fn deep_copy_mesh(mesh: &PolyData) -> PolyData {
    mesh.clone()
}

/// Append multiple meshes into one.
pub fn append_meshes(meshes: &[&PolyData]) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let mut verts = CellArray::new();

    for mesh in meshes {
        let offset = pts.len() as i64;
        for i in 0..mesh.points.len() {
            pts.push(mesh.points.get(i));
        }
        for cell in mesh.polys.iter() {
            let shifted: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
            polys.push_cell(&shifted);
        }
        for cell in mesh.lines.iter() {
            let shifted: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
            lines.push_cell(&shifted);
        }
        for cell in mesh.verts.iter() {
            let shifted: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
            verts.push_cell(&shifted);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result.lines = lines;
    result.verts = verts;
    result
}

/// Duplicate mesh N times with given offset between copies.
pub fn duplicate_mesh(mesh: &PolyData, n: usize, offset: [f64; 3]) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for copy in 0..n {
        let pt_offset = pts.len() as i64;
        let dx = offset[0] * copy as f64;
        let dy = offset[1] * copy as f64;
        let dz = offset[2] * copy as f64;
        for i in 0..mesh.points.len() {
            let p = mesh.points.get(i);
            pts.push([p[0] + dx, p[1] + dy, p[2] + dz]);
        }
        for cell in mesh.polys.iter() {
            let shifted: Vec<i64> = cell.iter().map(|&v| v + pt_offset).collect();
            polys.push_cell(&shifted);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_append() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[5.0, 5.0, 5.0], [6.0, 5.0, 5.0], [5.5, 6.0, 5.0]],
            vec![[0, 1, 2]],
        );
        let r = append_meshes(&[&a, &b]);
        assert_eq!(r.points.len(), 6);
        assert_eq!(r.polys.num_cells(), 2);
    }
    #[test]
    fn test_duplicate() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = duplicate_mesh(&mesh, 3, [2.0, 0.0, 0.0]);
        assert_eq!(r.points.len(), 9);
        assert_eq!(r.polys.num_cells(), 3);
        let p = r.points.get(3);
        assert!((p[0] - 2.0).abs() < 1e-10); // second copy offset by 2
    }
}
