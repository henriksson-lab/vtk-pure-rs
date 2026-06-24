//! Cell and point extraction from UnstructuredGrid.

use crate::data::{AnyDataArray, DataArray, Points, UnstructuredGrid};
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
    let mut selected: Vec<(CellType, Vec<i64>)> = Vec::new();

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
        selected.push((ct, new_ids));
    }

    let mut result = UnstructuredGrid::new();
    result.points = new_points;
    for (ct, ids) in &selected {
        result.push_cell(*ct, ids);
    }

    // Transfer point data for selected points
    let pd = grid.point_data();
    for ai in 0..pd.num_arrays() {
        if let Some(arr) = pd.get_array_by_index(ai) {
            let nc = arr.num_components();
            let name = arr.name().to_string();
            let mut data = Vec::new();
            let mut buf = vec![0.0f64; nc];
            let mut sorted_map: Vec<(usize, usize)> =
                point_map.iter().map(|(&o, &n)| (n, o)).collect();
            sorted_map.sort_by_key(|&(new, _)| new);
            for &(_, old) in &sorted_map {
                arr.tuple_as_f64(old, &mut buf);
                data.extend_from_slice(&buf);
            }
            result
                .point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(&name, data, nc)));
        }
    }

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
