//! Advanced mesh repair: close gaps, fill T-junctions, remove non-manifold
//! configurations, and ensure consistent orientation.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

/// Comprehensive mesh repair pipeline.
///
/// Applies: remove degenerate → merge close vertices → fix winding → remove small components.
pub fn repair_mesh(mesh: &PolyData, merge_tolerance: f64, min_component_faces: usize) -> PolyData {
    let step1 = remove_degenerate_triangles(mesh);
    let step2 = merge_close_vertices_simple(&step1, merge_tolerance);
    let step3 = super::orient_faces::orient_faces_consistent(&step2);
    if min_component_faces > 0 {
        remove_small_components_simple(&step3, min_component_faces)
    } else {
        step3
    }
}

/// Remove degenerate triangles (zero area or duplicate vertices).
pub fn remove_degenerate_triangles(mesh: &PolyData) -> PolyData {
    let mut polys = CellArray::new();
    let mut kept_polys = Vec::new();
    for (poly_id, cell) in mesh.polys.iter().enumerate() {
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
        kept_polys.push(poly_id);
    }
    let mut result = mesh.clone();
    result.polys = polys;
    remap_cell_data_for_kept_polys(mesh, &kept_polys, &mut result);
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
    let mut new_point_old_ids = Vec::new();
    let mut merged_to: Vec<Option<usize>> = vec![None; n];

    for i in 0..n {
        if merged_to[i].is_some() {
            continue;
        }
        let pi = mesh.points.get(i);
        let new_idx = new_points.len();
        new_points.push(pi);
        new_point_old_ids.push(i);
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

    let vert_offset = 0;
    let line_offset = mesh.verts.num_cells();
    let poly_offset = line_offset + mesh.lines.num_cells();
    let strip_offset = poly_offset + mesh.polys.num_cells();

    let mut kept_cells = Vec::new();
    let (verts, kept) = remap_cell_array_with_kept(&mesh.verts, &mapping, 1, vert_offset, n);
    kept_cells.extend(kept);
    let (lines, kept) = remap_cell_array_with_kept(&mesh.lines, &mapping, 2, line_offset, n);
    kept_cells.extend(kept);
    let (polys, kept) = remap_cell_array_with_kept(&mesh.polys, &mapping, 3, poly_offset, n);
    kept_cells.extend(kept);
    let (strips, kept) = remap_cell_array_with_kept(&mesh.strips, &mapping, 3, strip_offset, n);
    kept_cells.extend(kept);

    let mut result = mesh.clone();
    result.points = new_points;
    result.polys = polys;
    result.lines = lines;
    result.verts = verts;
    result.strips = strips;
    remap_point_data_by_indices(mesh, &new_point_old_ids, &mut result);
    remap_cell_data_by_indices(mesh, &kept_cells, &mut result);
    result
}

/// Remove connected components with fewer than min_faces faces.
fn remove_small_components_simple(mesh: &PolyData, min_faces: usize) -> PolyData {
    let n_cells = mesh.polys.num_cells();
    if n_cells == 0 {
        return mesh.clone();
    }

    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let valid_cells: Vec<bool> = all_cells
        .iter()
        .map(|cell| cell.len() >= 3 && cell_ids_are_valid(cell, mesh.points.len()))
        .collect();

    // Build adjacency via shared edges
    let mut edge_to_cells: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        if !valid_cells[ci] {
            continue;
        }
        let nc = cell.len();
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
        if labels[seed] != usize::MAX || !valid_cells[seed] {
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
    let mut new_lines = CellArray::new();
    let mut new_verts = CellArray::new();
    let mut new_strips = CellArray::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut used_old_points = Vec::new();
    let mut kept_cell_ids = Vec::new();

    copy_cells_compacted(
        &mesh.verts,
        mesh,
        &mut new_verts,
        &mut pt_map,
        &mut new_points,
        &mut used_old_points,
        &mut kept_cell_ids,
        0,
    );
    copy_cells_compacted(
        &mesh.lines,
        mesh,
        &mut new_lines,
        &mut pt_map,
        &mut new_points,
        &mut used_old_points,
        &mut kept_cell_ids,
        mesh.verts.num_cells(),
    );

    let poly_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    for (ci, cell) in all_cells.iter().enumerate() {
        if !valid_cells[ci] {
            continue;
        }
        if comp_sizes[labels[ci]] < min_faces {
            continue;
        }
        let mut new_ids = Vec::new();
        for &pid in cell {
            let old = pid as usize;
            let idx = *pt_map.entry(old).or_insert_with(|| {
                let i = new_points.len();
                new_points.push(mesh.points.get(old));
                used_old_points.push(old);
                i
            });
            new_ids.push(idx as i64);
        }
        new_polys.push_cell(&new_ids);
        kept_cell_ids.push(poly_offset + ci);
    }

    copy_cells_compacted(
        &mesh.strips,
        mesh,
        &mut new_strips,
        &mut pt_map,
        &mut new_points,
        &mut used_old_points,
        &mut kept_cell_ids,
        poly_offset + mesh.polys.num_cells(),
    );

    let mut result = mesh.clone();
    result.points = new_points;
    result.polys = new_polys;
    result.lines = new_lines;
    result.verts = new_verts;
    result.strips = new_strips;
    remap_point_data_by_indices(mesh, &used_old_points, &mut result);
    remap_cell_data_by_indices(mesh, &kept_cell_ids, &mut result);
    result
}

fn cell_ids_are_valid(cell: &[i64], num_points: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < num_points)
}

fn remap_cell_array_with_kept(
    cells: &CellArray,
    point_mapping: &[usize],
    min_unique: usize,
    global_offset: usize,
    num_points: usize,
) -> (CellArray, Vec<usize>) {
    let mut remapped_cells = CellArray::new();
    let mut kept = Vec::new();

    for (cell_id, cell) in cells.iter().enumerate() {
        if !cell_ids_are_valid(cell, num_points) {
            continue;
        }
        let new_ids: Vec<i64> = cell
            .iter()
            .map(|&id| point_mapping[id as usize] as i64)
            .collect();
        let mut unique = new_ids.clone();
        unique.sort_unstable();
        unique.dedup();
        if unique.len() >= min_unique {
            remapped_cells.push_cell(&new_ids);
            kept.push(global_offset + cell_id);
        }
    }

    (remapped_cells, kept)
}

fn copy_cells_compacted(
    cells: &CellArray,
    mesh: &PolyData,
    output: &mut CellArray,
    point_map: &mut std::collections::HashMap<usize, usize>,
    output_points: &mut Points<f64>,
    used_old_points: &mut Vec<usize>,
    kept_cell_ids: &mut Vec<usize>,
    global_offset: usize,
) {
    for (cell_id, cell) in cells.iter().enumerate() {
        if !cell_ids_are_valid(cell, mesh.points.len()) {
            continue;
        }
        let mut new_ids = Vec::with_capacity(cell.len());
        for &pid in cell {
            let old = pid as usize;
            let idx = *point_map.entry(old).or_insert_with(|| {
                let i = output_points.len();
                output_points.push(mesh.points.get(old));
                used_old_points.push(old);
                i
            });
            new_ids.push(idx as i64);
        }
        output.push_cell(&new_ids);
        kept_cell_ids.push(global_offset + cell_id);
    }
}

fn remap_point_data_by_indices(input: &PolyData, kept_points: &[usize], output: &mut PolyData) {
    output.point_data_mut().clear();
    for array in input.point_data().iter() {
        if array.num_tuples() == input.points.len() {
            output
                .point_data_mut()
                .add_array(select_tuples_by_indices(array, kept_points));
        }
    }
    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn remap_cell_data_for_kept_polys(input: &PolyData, kept_polys: &[usize], output: &mut PolyData) {
    let total_cells = input.total_cells();
    let poly_offset = input.verts.num_cells() + input.lines.num_cells();
    let mut kept = Vec::with_capacity(
        input.verts.num_cells()
            + input.lines.num_cells()
            + kept_polys.len()
            + input.strips.num_cells(),
    );

    kept.extend(0..poly_offset);
    kept.extend(kept_polys.iter().map(|&poly_id| poly_offset + poly_id));
    let strip_offset = poly_offset + input.polys.num_cells();
    kept.extend(strip_offset..total_cells);

    remap_cell_data_by_indices(input, &kept, output);
}

fn remap_cell_data_by_indices(input: &PolyData, kept_cells: &[usize], output: &mut PolyData) {
    let total_cells = input.total_cells();
    output.cell_data_mut().clear();
    for array in input.cell_data().iter() {
        if array.num_tuples() == total_cells {
            output
                .cell_data_mut()
                .add_array(select_tuples_by_indices(array, kept_cells));
        }
    }
    copy_active_attributes(input.cell_data(), output.cell_data_mut());
}

fn select_tuples_by_indices(array: &AnyDataArray, kept: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(select_typed_tuples_by_indices($array, kept))
        };
    }

    match array {
        AnyDataArray::F32(array) => select!(array, F32),
        AnyDataArray::F64(array) => select!(array, F64),
        AnyDataArray::I8(array) => select!(array, I8),
        AnyDataArray::I16(array) => select!(array, I16),
        AnyDataArray::I32(array) => select!(array, I32),
        AnyDataArray::I64(array) => select!(array, I64),
        AnyDataArray::U8(array) => select!(array, U8),
        AnyDataArray::U16(array) => select!(array, U16),
        AnyDataArray::U32(array) => select!(array, U32),
        AnyDataArray::U64(array) => select!(array, U64),
    }
}

fn select_typed_tuples_by_indices<T: Scalar>(array: &DataArray<T>, kept: &[usize]) -> DataArray<T> {
    let mut data = Vec::with_capacity(kept.len() * array.num_components());
    for &tuple_id in kept {
        data.extend_from_slice(array.tuple(tuple_id));
    }
    DataArray::from_vec(array.name(), data, array.num_components())
}

fn copy_active_attributes(input: &DataSetAttributes, output: &mut DataSetAttributes) {
    if let Some(array) = input.scalars() {
        output.set_active_scalars(array.name());
    }
    if let Some(array) = input.vectors() {
        output.set_active_vectors(array.name());
    }
    if let Some(array) = input.normals() {
        output.set_active_normals(array.name());
    }
    if let Some(array) = input.tcoords() {
        output.set_active_tcoords(array.name());
    }
    if let Some(array) = input.tensors() {
        output.set_active_tensors(array.name());
    }
    if let Some(array) = input.global_ids() {
        output.set_active_global_ids(array.name());
    }
    if let Some(array) = input.pedigree_ids() {
        output.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = input.edge_flags() {
        output.set_active_edge_flags(array.name());
    }
    if let Some(array) = input.tangents() {
        output.set_active_tangents(array.name());
    }
    if let Some(array) = input.rational_weights() {
        output.set_active_rational_weights(array.name());
    }
    if let Some(array) = input.higher_order_degrees() {
        output.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = input.process_ids() {
        output.set_active_process_ids(array.name());
    }
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
    fn full_repair_fixes_winding() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 2, 3]],
        );
        let result = repair_mesh(&mesh, 0.0, 0);
        let cells: Vec<Vec<i64>> = result.polys.iter().map(|c| c.to_vec()).collect();
        assert_eq!(cells, vec![vec![0, 1, 2], vec![3, 2, 1]]);
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
