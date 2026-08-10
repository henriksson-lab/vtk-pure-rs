//! Robust mesh boolean operations built on point-in-mesh classification.
//!
//! Provides union and intersection operations that classify whole cells by a
//! majority vote over their vertices; the difference operation is re-exported
//! from [`crate::filters::mesh::boolean_difference`].

use crate::data::{CellArray, Points, PolyData};

/// Classify each vertex of mesh A as inside/outside mesh B.
///
/// Boolean view of the single implementation in
/// [`crate::filters::mesh::mesh_mesh_boolean_classify::classify_vertices`],
/// which attaches the same classification as an "InsideRef" point array.
pub fn classify_vertices(mesh_a: &PolyData, mesh_b: &PolyData) -> Vec<bool> {
    let n = mesh_a.points.len();
    let classified =
        crate::filters::mesh::mesh_mesh_boolean_classify::classify_vertices(mesh_a, mesh_b);
    let Some(arr) = classified.point_data().get_array("InsideRef") else {
        return vec![false; n];
    };
    let mut buf = [0.0f64];
    (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] > 0.5
        })
        .collect()
}

/// Boolean union: keep faces from A outside B + faces from B outside A.
pub fn boolean_union(a: &PolyData, b: &PolyData) -> PolyData {
    let inside_b = classify_vertices(a, b);
    let inside_a = classify_vertices(b, a);

    let mut result_a = extract_cells_by_vertex_flag(a, &inside_b, false); // outside B
    let result_b = extract_cells_by_vertex_flag(b, &inside_a, false); // outside A

    append_mesh(&mut result_a, &result_b);
    result_a
}

/// Boolean intersection: keep faces from A inside B + faces from B inside A.
pub fn boolean_intersection(a: &PolyData, b: &PolyData) -> PolyData {
    let inside_b = classify_vertices(a, b);
    let inside_a = classify_vertices(b, a);

    let mut result_a = extract_cells_by_vertex_flag(a, &inside_b, true); // inside B
    let result_b = extract_cells_by_vertex_flag(b, &inside_a, true); // inside A

    append_mesh(&mut result_a, &result_b);
    result_a
}

/// Boolean difference: A minus B.
///
/// Re-exported from [`crate::filters::mesh::boolean_difference`], which holds
/// the single implementation (per-cell centroid classification, matching
/// `vtkBooleanOperationPolyDataFilter`).
pub use crate::filters::mesh::boolean_difference::boolean_difference;

fn extract_cells_by_vertex_flag(mesh: &PolyData, flags: &[bool], keep_flagged: bool) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let npts = mesh.points.len();

    for cell in mesh.polys.iter() {
        if cell.len() < 3 || cell.iter().any(|&pid| pid < 0 || pid as usize >= npts) {
            continue;
        }
        let majority = cell
            .iter()
            .filter(|&&pid| {
                let idx = pid as usize;
                idx < flags.len() && flags[idx]
            })
            .count()
            > cell.len() / 2;

        if majority != keep_flagged {
            continue;
        }

        let mut ids = Vec::new();
        for &pid in cell {
            let old = pid as usize;
            let new_idx = *pt_map.entry(old).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(old));
                i
            });
            ids.push(new_idx as i64);
        }
        polys.push_cell(&ids);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

fn append_mesh(target: &mut PolyData, source: &PolyData) {
    let offset = target.points.len() as i64;
    for i in 0..source.points.len() {
        target.points.push(source.points.get(i));
    }
    for cell in source.polys.iter() {
        let shifted: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
        target.polys.push_cell(&shifted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cube(cx: f64, cy: f64, cz: f64, size: f64) -> PolyData {
        let s = size / 2.0;
        PolyData::from_triangles(
            vec![
                [cx - s, cy - s, cz - s],
                [cx + s, cy - s, cz - s],
                [cx + s, cy + s, cz - s],
                [cx - s, cy + s, cz - s],
                [cx - s, cy - s, cz + s],
                [cx + s, cy - s, cz + s],
                [cx + s, cy + s, cz + s],
                [cx - s, cy + s, cz + s],
            ],
            vec![
                [0, 2, 1],
                [0, 3, 2],
                [4, 5, 6],
                [4, 6, 7],
                [0, 1, 5],
                [0, 5, 4],
                [2, 3, 7],
                [2, 7, 6],
                [0, 4, 7],
                [0, 7, 3],
                [1, 2, 6],
                [1, 6, 5],
            ],
        )
    }

    #[test]
    fn classify_inside() {
        let cube = make_cube(0.0, 0.0, 0.0, 2.0);
        let probe = PolyData::from_points(vec![[0.0, 0.0, 0.0], [5.0, 5.0, 5.0]]);
        let flags = classify_vertices(&probe, &cube);
        assert!(flags[0], "center should be inside");
        assert!(!flags[1], "far point should be outside");
    }

    #[test]
    fn union_two_cubes() {
        let a = make_cube(0.0, 0.0, 0.0, 2.0);
        let b = make_cube(1.0, 0.0, 0.0, 2.0);
        let result = boolean_union(&a, &b);
        assert!(result.polys.num_cells() > 0);
    }

    #[test]
    fn no_overlap() {
        let a = make_cube(0.0, 0.0, 0.0, 1.0);
        let b = make_cube(10.0, 0.0, 0.0, 1.0);
        let inter = boolean_intersection(&a, &b);
        assert_eq!(inter.polys.num_cells(), 0);
    }
}
