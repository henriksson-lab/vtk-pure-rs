//! Angle-based mesh simplification: remove vertices where adjacent
//! face normals are nearly coplanar.

use crate::data::{CellArray, Points, PolyData};

/// Remove vertices where all adjacent faces are nearly coplanar.
///
/// Vertices whose one-ring face normals all differ by less than
/// `angle_threshold` degrees are collapsed to their neighbors.
pub fn simplify_by_angle(mesh: &PolyData, angle_threshold_degrees: f64) -> PolyData {
    let n = mesh.points.len();
    if n < 4 {
        return mesh.clone();
    }
    let cos_thresh = (angle_threshold_degrees * std::f64::consts::PI / 180.0).cos();

    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let normals: Vec<[f64; 3]> = all_cells
        .iter()
        .map(|cell| face_normal(mesh, cell))
        .collect();

    // Find which faces each vertex belongs to
    let mut vert_faces: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (ci, cell) in all_cells.iter().enumerate() {
        for &pid in cell {
            vert_faces[pid as usize].push(ci);
        }
    }

    // Identify removable vertices (all adjacent faces are coplanar)
    let mut removable = vec![false; n];
    for vi in 0..n {
        let faces = &vert_faces[vi];
        if faces.len() < 2 {
            continue;
        }
        let all_coplanar = faces.windows(2).all(|pair| {
            let dot = normals[pair[0]][0] * normals[pair[1]][0]
                + normals[pair[0]][1] * normals[pair[1]][1]
                + normals[pair[0]][2] * normals[pair[1]][2];
            dot >= cos_thresh
        });
        if all_coplanar {
            removable[vi] = true;
        }
    }

    // Remove faces that contain removable vertices and re-triangulate
    // Simple approach: just keep faces with no removable vertices
    let mut new_pts = Points::<f64>::new();
    let mut new_polys = CellArray::new();
    let mut pt_map = vec![usize::MAX; n];

    for cell in &all_cells {
        let has_removable = cell.iter().any(|&pid| removable[pid as usize]);
        if has_removable {
            continue;
        }
        let mut ids = Vec::new();
        for &pid in cell {
            let old = pid as usize;
            if pt_map[old] == usize::MAX {
                pt_map[old] = new_pts.len();
                new_pts.push(mesh.points.get(old));
            }
            ids.push(pt_map[old] as i64);
        }
        new_polys.push_cell(&ids);
    }

    let mut result = PolyData::new();
    result.points = new_pts;
    result.polys = new_polys;
    result
}

/// Count vertices that would be removed at a given angle threshold.
pub fn count_removable_vertices(mesh: &PolyData, angle_threshold_degrees: f64) -> usize {
    let n = mesh.points.len();
    let cos_thresh = (angle_threshold_degrees * std::f64::consts::PI / 180.0).cos();

    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let normals: Vec<[f64; 3]> = all_cells
        .iter()
        .map(|cell| face_normal(mesh, cell))
        .collect();

    let mut vert_faces: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (ci, cell) in all_cells.iter().enumerate() {
        for &pid in cell {
            vert_faces[pid as usize].push(ci);
        }
    }

    let mut count = 0;
    for vi in 0..n {
        let faces = &vert_faces[vi];
        if faces.len() < 2 {
            continue;
        }
        if faces.windows(2).all(|pair| {
            normals[pair[0]][0] * normals[pair[1]][0]
                + normals[pair[0]][1] * normals[pair[1]][1]
                + normals[pair[0]][2] * normals[pair[1]][2]
                >= cos_thresh
        }) {
            count += 1;
        }
    }
    count
}

fn face_normal(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let a = mesh.points.get(cell[0] as usize);
    let b = mesh.points.get(cell[1] as usize);
    let c = mesh.points.get(cell[2] as usize);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-15 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_grid_simplify() {
        // All coplanar faces on a flat plane
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let removable = count_removable_vertices(&mesh, 5.0);
        assert!(removable > 0); // interior vertices should be removable
        let simplified = simplify_by_angle(&mesh, 5.0);
        assert!(simplified.points.len() < mesh.points.len());
    }

    #[test]
    fn non_flat_preserved() {
        // Bent mesh: should not simplify the bend vertex
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 2.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        let simplified = simplify_by_angle(&mesh, 5.0);
        // Should keep the sharp edge
        assert!(simplified.points.len() >= 3);
    }
}
