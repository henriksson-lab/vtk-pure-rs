//! Progressive decimation with feature preservation.

use crate::data::{CellArray, Points, PolyData};

/// Decimate mesh to target reduction ratio while preserving boundary and feature edges.
/// `target_reduction` is the fraction of triangles to remove (0.0 to 1.0).
/// `feature_angle` in degrees — edges with dihedral angle above this are preserved.
pub fn decimate_pro(mesh: &PolyData, target_reduction: f64, feature_angle: f64) -> PolyData {
    let num_cells = mesh.polys.num_cells();
    if num_cells == 0 {
        return mesh.clone();
    }

    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let npts = mesh.points.len();
    let target_remove = (num_cells as f64 * target_reduction.clamp(0.0, 0.99)) as usize;
    let cos_feature = (feature_angle.to_radians()).cos();

    // Build edge-face adjacency
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in cells.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    // Find boundary and feature vertices
    let mut protected = vec![false; npts];
    for (&(_a, _b), faces) in &edge_faces {
        if faces.len() == 1 {
            // Boundary edge — protect both vertices
            protected[_a] = true;
            protected[_b] = true;
        } else if faces.len() == 2 {
            let n0 = face_normal(&cells[faces[0]], &mesh.points);
            let n1 = face_normal(&cells[faces[1]], &mesh.points);
            let dot = n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2];
            if dot < cos_feature {
                protected[_a] = true;
                protected[_b] = true;
            }
        }
    }

    // Vertex-to-face adjacency
    let mut vert_faces: Vec<Vec<usize>> = vec![Vec::new(); npts];
    for (ci, cell) in cells.iter().enumerate() {
        for &v in cell {
            vert_faces[v as usize].push(ci);
        }
    }

    // Greedy vertex removal
    let mut removed_cells = vec![false; num_cells];
    let mut collapsed_to: Vec<Option<usize>> = vec![None; npts]; // vertex -> replacement
    let mut removals = 0;

    // Sort vertices by valence (low valence first for simplicity)
    let mut verts_by_valence: Vec<usize> = (0..npts).collect();
    verts_by_valence.sort_by_key(|&v| vert_faces[v].len());

    for &v in &verts_by_valence {
        if removals >= target_remove {
            break;
        }
        if protected[v] {
            continue;
        }
        if vert_faces[v].is_empty() {
            continue;
        }

        // Find best neighbor to collapse to
        let mut neighbors: Vec<usize> = Vec::new();
        for &fi in &vert_faces[v] {
            if removed_cells[fi] {
                continue;
            }
            for &vid in &cells[fi] {
                let vid = resolve(vid as usize, &collapsed_to);
                if vid != v && !neighbors.contains(&vid) {
                    neighbors.push(vid);
                }
            }
        }
        if neighbors.is_empty() {
            continue;
        }

        // Pick closest neighbor
        let vp = mesh.points.get(v);
        let target = *neighbors
            .iter()
            .min_by(|&&a, &&b| {
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                let da = dist_sq(vp, pa);
                let db = dist_sq(vp, pb);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        // Remove degenerate faces
        let mut faces_removed = 0;
        for &fi in &vert_faces[v] {
            if removed_cells[fi] {
                continue;
            }
            let resolved: Vec<usize> = cells[fi]
                .iter()
                .map(|&vid| resolve(vid as usize, &collapsed_to))
                .collect();
            // If collapsing v->target makes this face degenerate
            let new_face: Vec<usize> = resolved
                .iter()
                .map(|&vid| if vid == v { target } else { vid })
                .collect();
            let unique: std::collections::HashSet<usize> = new_face.iter().copied().collect();
            if unique.len() < 3 {
                removed_cells[fi] = true;
                faces_removed += 1;
            }
        }
        if faces_removed > 0 {
            collapsed_to[v] = Some(target);
            removals += faces_removed;
        }
    }

    // Rebuild mesh
    let mut new_pts = Points::<f64>::new();
    let mut pt_map = vec![usize::MAX; npts];
    for i in 0..npts {
        if collapsed_to[i].is_none() {
            pt_map[i] = new_pts.len();
            new_pts.push(mesh.points.get(i));
        }
    }
    // Map collapsed vertices
    for i in 0..npts {
        if let Some(_) = collapsed_to[i] {
            let target = resolve(i, &collapsed_to);
            pt_map[i] = pt_map[target];
        }
    }

    let mut new_polys = CellArray::new();
    for (ci, cell) in cells.iter().enumerate() {
        if removed_cells[ci] {
            continue;
        }
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        let unique: std::collections::HashSet<i64> = mapped.iter().copied().collect();
        if unique.len() >= 3 {
            new_polys.push_cell(&mapped);
        }
    }

    let mut result = PolyData::new();
    result.points = new_pts;
    result.polys = new_polys;
    result
}

fn resolve(mut v: usize, collapsed_to: &[Option<usize>]) -> usize {
    while let Some(next) = collapsed_to[v] {
        v = next;
    }
    v
}

fn dist_sq(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

fn face_normal(cell: &[i64], points: &crate::data::Points<f64>) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let a = points.get(cell[0] as usize);
    let b = points.get(cell[1] as usize);
    let c = points.get(cell[2] as usize);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [n[0] / len, n[1] / len, n[2] / len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decimate_pro() {
        // 4 triangles in a fan
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3], [0, 3, 4], [0, 4, 1]],
        );
        let result = decimate_pro(&mesh, 0.5, 30.0);
        assert!(result.polys.num_cells() <= 4);
        assert!(result.polys.num_cells() >= 1);
    }
    #[test]
    fn test_no_decimate() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = decimate_pro(&mesh, 0.0, 30.0);
        assert_eq!(result.polys.num_cells(), 1);
    }
}
