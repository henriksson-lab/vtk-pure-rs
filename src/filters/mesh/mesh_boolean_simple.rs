//! Simple mesh boolean operations using point classification.

use crate::data::{CellArray, Points, PolyData};

/// Classify each vertex of mesh A as inside or outside mesh B using ray casting.
///
/// Thin wrapper over [`crate::filters::mesh::mesh_point_containment::classify_points`],
/// which takes the enclosing surface first and the query points as a slice.
pub fn classify_points(mesh: &PolyData, reference: &PolyData) -> Vec<bool> {
    let points: Vec<[f64; 3]> = (0..mesh.points.len()).map(|i| mesh.points.get(i)).collect();
    crate::filters::mesh::mesh_point_containment::classify_points(reference, &points)
}

/// Extract faces of mesh A that are inside mesh B.
pub fn extract_inside(mesh_a: &PolyData, mesh_b: &PolyData) -> PolyData {
    extract_classified(mesh_a, mesh_b, true)
}

/// Extract faces of mesh A that are outside mesh B.
pub fn extract_outside(mesh_a: &PolyData, mesh_b: &PolyData) -> PolyData {
    extract_classified(mesh_a, mesh_b, false)
}

fn extract_classified(mesh: &PolyData, reference: &PolyData, want_inside: bool) -> PolyData {
    let inside = classify_points(mesh, reference);
    let mut used = vec![false; mesh.points.len()];
    let mut kept = Vec::new();

    for cell in mesh.polys.iter() {
        if cell.is_empty() || !valid_cell(cell, mesh.points.len()) {
            continue;
        }
        let all_match = cell.iter().all(|&v| inside[v as usize] == want_inside);
        if all_match {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.to_vec());
        }
    }

    let mut pt_map = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for cell in &kept {
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        polys.push_cell(&mapped);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

fn valid_cell(cell: &[i64], npoints: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < npoints)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tetrahedron() -> PolyData {
        PolyData::from_triangles(
            vec![
                [-5.0, -5.0, -5.0],
                [5.0, -5.0, -5.0],
                [0.0, 5.0, -5.0],
                [0.0, 0.0, 5.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        )
    }

    #[test]
    fn classify_marks_enclosed_vertices() {
        let big = tetrahedron();
        let probe = PolyData::from_points(vec![[0.0, 0.0, 0.0], [20.0, 0.0, 0.0]]);
        let flags = classify_points(&probe, &big);
        assert_eq!(flags, vec![true, false]);
    }

    #[test]
    fn extract_inside_keeps_only_enclosed_faces() {
        let big = tetrahedron();
        let inner = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.5, 0.0, 0.0], [0.0, 0.5, 0.0]],
            vec![[0, 1, 2]],
        );
        assert_eq!(extract_inside(&inner, &big).polys.num_cells(), 1);
        assert_eq!(extract_outside(&inner, &big).polys.num_cells(), 0);
    }
}
