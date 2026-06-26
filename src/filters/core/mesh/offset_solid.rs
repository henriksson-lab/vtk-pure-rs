//! Create solid (thick) meshes by offsetting surfaces inward/outward
//! and connecting the two shells with side faces.

use crate::data::{CellArray, Points, PolyData};

/// Create a solid mesh by offsetting the surface along normals.
///
/// Produces an outer shell, an inner shell (reversed), and side quads
/// connecting boundary edges. The result is a closed solid.
pub fn offset_to_solid(mesh: &PolyData, thickness: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    let normals = compute_vertex_normals(mesh);
    let mut pts = Points::<f64>::new();

    // Outer shell (original + offset)
    for i in 0..n {
        let p = mesh.points.get(i);
        let nm = &normals[i];
        pts.push([
            p[0] + nm[0] * thickness * 0.5,
            p[1] + nm[1] * thickness * 0.5,
            p[2] + nm[2] * thickness * 0.5,
        ]);
    }
    // Inner shell (original - offset)
    for i in 0..n {
        let p = mesh.points.get(i);
        let nm = &normals[i];
        pts.push([
            p[0] - nm[0] * thickness * 0.5,
            p[1] - nm[1] * thickness * 0.5,
            p[2] - nm[2] * thickness * 0.5,
        ]);
    }

    let mut polys = CellArray::new();

    // Outer shell (same winding)
    for cell in mesh.polys.iter() {
        if !cell_point_ids_are_valid(cell, n) {
            continue;
        }
        polys.push_cell(cell);
    }
    // Inner shell (reversed winding)
    for cell in mesh.polys.iter() {
        if !cell_point_ids_are_valid(cell, n) {
            continue;
        }
        let reversed: Vec<i64> = cell.iter().rev().map(|&id| id + n as i64).collect();
        polys.push_cell(&reversed);
    }

    // Side faces on boundary edges
    let boundary = find_boundary_edges(mesh);
    for (a, b) in &boundary {
        let a0 = *a as i64;
        let b0 = *b as i64;
        let a1 = a0 + n as i64;
        let b1 = b0 + n as i64;
        polys.push_cell(&[a0, b0, b1]);
        polys.push_cell(&[a0, b1, a1]);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

fn compute_vertex_normals(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut normals = vec![[0.0; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let Some(a_idx) = valid_point_id(cell[0], n) else {
            continue;
        };
        let Some(b_idx) = valid_point_id(cell[1], n) else {
            continue;
        };
        let Some(c_idx) = valid_point_id(cell[2], n) else {
            continue;
        };
        let a = mesh.points.get(a_idx);
        let b = mesh.points.get(b_idx);
        let c = mesh.points.get(c_idx);
        let fn_ = [
            (b[1] - a[1]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[1] - a[1]),
            (b[2] - a[2]) * (c[0] - a[0]) - (b[0] - a[0]) * (c[2] - a[2]),
            (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]),
        ];
        for &pid in cell {
            if let Some(idx) = valid_point_id(pid, n) {
                for c in 0..3 {
                    normals[idx][c] += fn_[c];
                }
            }
        }
    }
    for nm in &mut normals {
        let len = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
        if len > 1e-15 {
            for c in 0..3 {
                nm[c] /= len;
            }
        }
    }
    normals
}

fn find_boundary_edges(mesh: &PolyData) -> Vec<(usize, usize)> {
    let mut edge_counts: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    let mut oriented_edges = Vec::new();
    let n = mesh.points.len();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], n) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], n) else {
                continue;
            };
            *edge_counts.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            oriented_edges.push((a, b));
        }
    }
    oriented_edges
        .into_iter()
        .filter(|&(a, b)| edge_counts[&(a.min(b), a.max(b))] == 1)
        .collect()
}

fn valid_point_id(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

fn cell_point_ids_are_valid(cell: &[i64], n_points: usize) -> bool {
    cell.iter().all(|&id| valid_point_id(id, n_points).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_tri_to_solid() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let solid = offset_to_solid(&mesh, 0.1);
        assert_eq!(solid.points.len(), 6); // 3 outer + 3 inner
        assert!(solid.polys.num_cells() > 2); // 2 shells + side faces
    }

    #[test]
    fn boundary_edges_keep_polygon_order() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[2, 1, 0]],
        );

        assert_eq!(find_boundary_edges(&mesh), vec![(2, 1), (1, 0), (0, 2)]);
    }

    #[test]
    fn closed_tet_no_sides() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
        );
        let solid = offset_to_solid(&mesh, 0.1);
        assert_eq!(solid.polys.num_cells(), 8); // 4 outer + 4 inner, no boundary
    }

    #[test]
    fn malformed_cells_do_not_panic() {
        let mut mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        mesh.polys.push_cell(&[0, -1, 99]);

        let solid = offset_to_solid(&mesh, 0.1);
        assert_eq!(solid.points.len(), 4);
        assert_eq!(solid.polys.num_cells(), 0);
    }
}
