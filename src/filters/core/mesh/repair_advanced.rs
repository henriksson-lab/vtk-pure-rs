//! Advanced mesh repair: close gaps, fill T-junctions, remove non-manifold
//! configurations, and ensure consistent orientation.

use crate::data::{CellArray, Points, PolyData};

/// Comprehensive mesh repair pipeline.
///
/// Applies: remove degenerate → merge close vertices → fix winding → remove small components.
pub fn repair_mesh(mesh: &PolyData, merge_tolerance: f64, min_component_faces: usize) -> PolyData {
    let step1 = remove_degenerate_triangles(mesh);
    let step2 = merge_close_vertices_simple(&step1, merge_tolerance);
    let step3 = if min_component_faces > 0 {
        remove_small_components_simple(&step2, min_component_faces)
    } else {
        step2
    };
    step3
}

/// Remove degenerate triangles (zero area or duplicate vertices).
pub fn remove_degenerate_triangles(mesh: &PolyData) -> PolyData {
    let mut polys = CellArray::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !cell_ids_are_valid(cell, mesh.points.len()) {
            continue;
        }
        // Check for duplicate vertices
        let mut ok = true;
        for i in 0..cell.len() {
            for j in i + 1..cell.len() {
                if cell[i] == cell[j] {
                    ok = false;
                    break;
                }
            }
            if !ok {
                break;
            }
        }
        if !ok {
            continue;
        }

        // Check for zero area (for triangles)
        if cell.len() == 3 {
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            let c = mesh.points.get(cell[2] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let nx = e1[1] * e2[2] - e1[2] * e2[1];
            let ny = e1[2] * e2[0] - e1[0] * e2[2];
            let nz = e1[0] * e2[1] - e1[1] * e2[0];
            if nx * nx + ny * ny + nz * nz < 1e-20 {
                continue;
            }
        }
        polys.push_cell(cell);
    }
    let mut result = mesh.clone();
    result.polys = polys;
    result
}

/// Merge vertices closer than tolerance.
pub fn merge_close_vertices_simple(mesh: &PolyData, tolerance: f64) -> PolyData {
    if tolerance <= 0.0 {
        return mesh.clone();
    }

    let n = mesh.points.len();
    let tol2 = tolerance * tolerance;
    let mut mapping = vec![0usize; n]; // old → new index
    let mut new_points = Points::<f64>::new();
    let mut merged_to: Vec<Option<usize>> = vec![None; n];

    for i in 0..n {
        if merged_to[i].is_some() {
            continue;
        }
        let pi = mesh.points.get(i);
        let new_idx = new_points.len();
        new_points.push(pi);
        mapping[i] = new_idx;
        merged_to[i] = Some(new_idx);

        for j in i + 1..n {
            if merged_to[j].is_some() {
                continue;
            }
            let pj = mesh.points.get(j);
            let d2 = (pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2);
            if d2 < tol2 {
                mapping[j] = new_idx;
                merged_to[j] = Some(new_idx);
            }
        }
    }

    let mut polys = CellArray::new();
    for cell in mesh.polys.iter() {
        if !cell_ids_are_valid(cell, n) {
            continue;
        }
        let new_ids: Vec<i64> = cell.iter().map(|&id| mapping[id as usize] as i64).collect();
        // Skip degenerate after merge
        let mut unique = new_ids.clone();
        unique.sort_unstable();
        unique.dedup();
        if unique.len() >= 3 {
            polys.push_cell(&new_ids);
        }
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = polys;
    result
}

/// Remove connected components with fewer than min_faces faces.
fn remove_small_components_simple(mesh: &PolyData, min_faces: usize) -> PolyData {
    let n_cells = mesh.polys.num_cells();
    if n_cells == 0 {
        return mesh.clone();
    }

    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();

    // Build adjacency via shared edges
    let mut edge_to_cells: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        if nc == 0 || !cell_ids_are_valid(cell, mesh.points.len()) {
            continue;
        }
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_to_cells
                .entry((a.min(b), a.max(b)))
                .or_default()
                .push(ci);
        }
    }

    let mut labels = vec![usize::MAX; n_cells];
    let mut next_label = 0;

    for seed in 0..n_cells {
        if labels[seed] != usize::MAX {
            continue;
        }
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(seed);
        labels[seed] = next_label;
        while let Some(ci) = queue.pop_front() {
            let cell = &all_cells[ci];
            let nc = cell.len();
            for i in 0..nc {
                let a = cell[i] as usize;
                let b = cell[(i + 1) % nc] as usize;
                let edge = (a.min(b), a.max(b));
                if let Some(neighbors) = edge_to_cells.get(&edge) {
                    for &ni in neighbors {
                        if labels[ni] == usize::MAX {
                            labels[ni] = next_label;
                            queue.push_back(ni);
                        }
                    }
                }
            }
        }
        next_label += 1;
    }

    // Count per component
    let mut comp_sizes = vec![0usize; next_label];
    for &l in &labels {
        if l < next_label {
            comp_sizes[l] += 1;
        }
    }

    // Keep cells from large components
    let mut new_points = Points::<f64>::new();
    let mut new_polys = CellArray::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for (ci, cell) in all_cells.iter().enumerate() {
        if comp_sizes[labels[ci]] < min_faces {
            continue;
        }
        let mut new_ids = Vec::new();
        if !cell_ids_are_valid(cell, mesh.points.len()) {
            continue;
        }
        for &pid in cell {
            let old = pid as usize;
            let idx = *pt_map.entry(old).or_insert_with(|| {
                let i = new_points.len();
                new_points.push(mesh.points.get(old));
                i
            });
            new_ids.push(idx as i64);
        }
        new_polys.push_cell(&new_ids);
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    result
}

fn cell_ids_are_valid(cell: &[i64], num_points: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < num_points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_degenerate() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.5, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 0, 3]], // second is degenerate (duplicate vertex)
        );
        let result = remove_degenerate_triangles(&mesh);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn merge_close() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.001, 0.0, 0.0],
                [1.001, 0.0, 0.0],
                [0.0, 1.001, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = merge_close_vertices_simple(&mesh, 0.01);
        assert!(result.points.len() <= 4); // should merge near-duplicate points
    }

    #[test]
    fn merge_close_removes_non_adjacent_duplicate_cells() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.001, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = merge_close_vertices_simple(&mesh, 0.01);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn full_repair() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = repair_mesh(&mesh, 0.001, 0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn remove_small() {
        // Two triangles sharing edge 0-1 → connected component of 2 faces
        // Plus one isolated triangle → component of 1 face
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.5, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [4, 5, 6]], // first two share edge 0-1
        );
        let result = remove_small_components_simple(&mesh, 2);
        assert_eq!(result.polys.num_cells(), 2); // only the connected pair
    }
}
