use crate::data::{Points, UnstructuredGrid};
use crate::types::CellType;

/// Extract cells of a specific type from an UnstructuredGrid.
pub fn extract_cells_by_type(input: &UnstructuredGrid, cell_type: CellType) -> UnstructuredGrid {
    extract_cells_by_types(input, &[cell_type])
}

/// Extract cells of any of the given types.
pub fn extract_cells_by_types(input: &UnstructuredGrid, types: &[CellType]) -> UnstructuredGrid {
    let mut used_points = vec![false; input.points.len()];
    let mut selected_cells = Vec::new();

    for i in 0..input.cells().num_cells() {
        if types.contains(&input.cell_type(i)) {
            selected_cells.push(i);
            for &pid in input.cell_points(i) {
                if (pid as usize) < used_points.len() {
                    used_points[pid as usize] = true;
                }
            }
        }
    }

    let mut old_to_new = vec![usize::MAX; input.points.len()];
    let mut new_points = Points::new();
    for (i, &used) in used_points.iter().enumerate() {
        if used {
            old_to_new[i] = new_points.len();
            new_points.push(input.points.get(i));
        }
    }

    let mut result = UnstructuredGrid::new();
    result.points = new_points;
    for &ci in &selected_cells {
        let pts: Vec<i64> = input
            .cell_points(ci)
            .iter()
            .map(|&pid| old_to_new[pid as usize] as i64)
            .collect();
        result.push_cell(input.cell_type(ci), &pts);
    }

    result
}

/// Extract only triangular cells as a new grid.
pub fn extract_triangles(input: &UnstructuredGrid) -> UnstructuredGrid {
    extract_cells_by_type(input, CellType::Triangle)
}

/// Extract only tetrahedral cells as a new grid.
pub fn extract_tetrahedra(input: &UnstructuredGrid) -> UnstructuredGrid {
    extract_cells_by_type(input, CellType::Tetra)
}

/// Get a summary of cell type counts.
pub fn cell_type_counts(input: &UnstructuredGrid) -> std::collections::HashMap<CellType, usize> {
    let mut counts = std::collections::HashMap::new();
    for i in 0..input.cells().num_cells() {
        *counts.entry(input.cell_type(i)).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mixed() -> UnstructuredGrid {
        let mut ug = UnstructuredGrid::new();
        for i in 0..6 {
            ug.points.push([i as f64, 0.0, 0.0]);
        }
        ug.push_cell(CellType::Triangle, &[0, 1, 2]);
        ug.push_cell(CellType::Quad, &[0, 1, 3, 4]);
        ug.push_cell(CellType::Triangle, &[3, 4, 5]);
        ug
    }

    #[test]
    fn extract_triangles_only() {
        let ug = make_mixed();
        let tris = extract_cells_by_type(&ug, CellType::Triangle);
        assert_eq!(tris.cells().num_cells(), 2);
    }

    #[test]
    fn extract_quads_only() {
        let ug = make_mixed();
        let quads = extract_cells_by_type(&ug, CellType::Quad);
        assert_eq!(quads.cells().num_cells(), 1);
    }

    #[test]
    fn cell_type_count() {
        let ug = make_mixed();
        let counts = cell_type_counts(&ug);
        assert_eq!(counts[&CellType::Triangle], 2);
        assert_eq!(counts[&CellType::Quad], 1);
    }

    #[test]
    fn extract_multiple_types() {
        let ug = make_mixed();
        let result = extract_cells_by_types(&ug, &[CellType::Triangle, CellType::Quad]);
        assert_eq!(result.cells().num_cells(), 3); // all cells
    }
}
