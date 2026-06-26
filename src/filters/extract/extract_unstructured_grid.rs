//! Cell and point extraction from UnstructuredGrid.

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, Points, UnstructuredGrid};
use crate::types::CellType;

/// Extract cells from an UnstructuredGrid by cell type.
pub fn extract_cells_by_type(grid: &UnstructuredGrid, cell_type: CellType) -> UnstructuredGrid {
    extract_cells_by_predicate(grid, |ct, _| ct == cell_type)
}

/// Extract cells from an UnstructuredGrid by predicate on (CellType, cell_index).
pub fn extract_cells_by_predicate(
    grid: &UnstructuredGrid,
    predicate: impl Fn(CellType, usize) -> bool,
) -> UnstructuredGrid {
    let mut new_points = Points::<f64>::new();
    let mut point_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    let types = grid.cell_types();
    let cells = grid.cells();

    // Collect cells that match
    let mut selected: Vec<(usize, CellType, Vec<i64>)> = Vec::new();

    for (ci, cell) in cells.iter().enumerate() {
        let ct = if ci < types.len() {
            types[ci]
        } else {
            CellType::Triangle
        };
        if !predicate(ct, ci) {
            continue;
        }

        let mut new_ids = Vec::with_capacity(cell.len());
        for &pid in cell {
            let old = pid as usize;
            let new_idx = *point_map.entry(old).or_insert_with(|| {
                let idx = new_points.len();
                new_points.push(grid.points.get(old));
                idx
            });
            new_ids.push(new_idx as i64);
        }
        selected.push((ci, ct, new_ids));
    }

    let mut result = UnstructuredGrid::new();
    result.points = new_points;
    for (_, ct, ids) in &selected {
        result.push_cell(*ct, ids);
    }

    // Transfer point data for selected points
    let pd = grid.point_data();
    for ai in 0..pd.num_arrays() {
        if let Some(arr) = pd.get_array_by_index(ai) {
            if arr.num_tuples() == grid.points.len() {
                let mut sorted_map: Vec<(usize, usize)> =
                    point_map.iter().map(|(&o, &n)| (n, o)).collect();
                sorted_map.sort_by_key(|&(new, _)| new);
                let tuple_ids: Vec<usize> = sorted_map.iter().map(|&(_, old)| old).collect();
                result
                    .point_data_mut()
                    .add_array(select_tuples(arr, &tuple_ids));
            }
        }
    }
    copy_active_attributes(grid.point_data(), result.point_data_mut());

    let selected_cell_ids: Vec<usize> = selected
        .iter()
        .map(|&(old_cell_id, _, _)| old_cell_id)
        .collect();
    for array in grid.cell_data().iter() {
        if array.num_tuples() == grid.cells().num_cells() {
            result
                .cell_data_mut()
                .add_array(select_tuples(array, &selected_cell_ids));
        }
    }
    copy_active_attributes(grid.cell_data(), result.cell_data_mut());

    result
}

/// Extract cells by index list.
pub fn extract_cells_by_indices(grid: &UnstructuredGrid, indices: &[usize]) -> UnstructuredGrid {
    let index_set: std::collections::HashSet<usize> = indices.iter().cloned().collect();
    extract_cells_by_predicate(grid, |_, ci| index_set.contains(&ci))
}

/// Count cells by type in an UnstructuredGrid.
pub fn cell_type_counts(grid: &UnstructuredGrid) -> std::collections::HashMap<CellType, usize> {
    let mut counts = std::collections::HashMap::new();
    for &ct in grid.cell_types() {
        *counts.entry(ct).or_insert(0) += 1;
    }
    counts
}

fn select_tuples(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {{
            let mut out = DataArray::new($array.name(), $array.num_components());
            for &tuple_id in tuple_ids {
                out.push_tuple($array.tuple(tuple_id));
            }
            AnyDataArray::$variant(out)
        }};
    }

    match array {
        AnyDataArray::F32(a) => select!(a, F32),
        AnyDataArray::F64(a) => select!(a, F64),
        AnyDataArray::I8(a) => select!(a, I8),
        AnyDataArray::I16(a) => select!(a, I16),
        AnyDataArray::I32(a) => select!(a, I32),
        AnyDataArray::I64(a) => select!(a, I64),
        AnyDataArray::U8(a) => select!(a, U8),
        AnyDataArray::U16(a) => select!(a, U16),
        AnyDataArray::U32(a) => select!(a, U32),
        AnyDataArray::U64(a) => select!(a, U64),
    }
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

    fn make_mixed_grid() -> UnstructuredGrid {
        let mut grid = UnstructuredGrid::new();
        grid.points = Points::from(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0], // triangle
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [3.0, 1.0, 0.0],
            [2.0, 1.0, 0.0], // quad
        ]);
        grid.push_cell(CellType::Triangle, &[0, 1, 2]);
        grid.push_cell(CellType::Quad, &[3, 4, 5, 6]);
        grid
    }

    #[test]
    fn extract_triangles() {
        let grid = make_mixed_grid();
        let result = extract_cells_by_type(&grid, CellType::Triangle);
        assert_eq!(result.cells().num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn extract_quads() {
        let grid = make_mixed_grid();
        let result = extract_cells_by_type(&grid, CellType::Quad);
        assert_eq!(result.cells().num_cells(), 1);
        assert_eq!(result.points.len(), 4);
    }

    #[test]
    fn extract_by_indices() {
        let grid = make_mixed_grid();
        let result = extract_cells_by_indices(&grid, &[0]);
        assert_eq!(result.cells().num_cells(), 1);
    }

    #[test]
    fn type_counts() {
        let grid = make_mixed_grid();
        let counts = cell_type_counts(&grid);
        assert_eq!(counts[&CellType::Triangle], 1);
        assert_eq!(counts[&CellType::Quad], 1);
    }
}
