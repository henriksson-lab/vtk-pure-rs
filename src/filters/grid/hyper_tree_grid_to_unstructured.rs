//! Convert HyperTreeGrid to UnstructuredGrid.

use crate::data::{HyperTreeGrid, UnstructuredGrid};
use crate::types::CellType;

/// Convert a HyperTreeGrid's coarse cells to an UnstructuredGrid.
///
/// Each coarse cell becomes a hexahedron (3D) or quad (2D).
pub fn hyper_tree_grid_to_unstructured_grid(htg: &HyperTreeGrid) -> UnstructuredGrid {
    let gs = htg.grid_size();
    let bounds = htg.grid_bounds();
    let spacing = [
        (bounds.x_max - bounds.x_min) / gs[0] as f64,
        (bounds.y_max - bounds.y_min) / gs[1] as f64,
        if gs[2] > 1 {
            (bounds.z_max - bounds.z_min) / gs[2] as f64
        } else {
            1.0
        },
    ];
    let origin = [bounds.x_min, bounds.y_min, bounds.z_min];

    let mut grid = UnstructuredGrid::new();

    if htg.dimensions()[2] <= 1 {
        // 2D: generate quads
        let mut pt_map: std::collections::HashMap<[i64; 2], usize> =
            std::collections::HashMap::new();
        for j in 0..gs[1] {
            for i in 0..gs[0] {
                let x0 = origin[0] + i as f64 * spacing[0];
                let y0 = origin[1] + j as f64 * spacing[1];
                let x1 = x0 + spacing[0];
                let y1 = y0 + spacing[1];

                let corners = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]];
                let mut ids = Vec::new();
                for c in &corners {
                    let key = [(c[0] * 1e6) as i64, (c[1] * 1e6) as i64];
                    let idx = *pt_map.entry(key).or_insert_with(|| {
                        let idx = grid.points.len();
                        grid.points.push([c[0], c[1], 0.0]);
                        idx
                    });
                    ids.push(idx as i64);
                }
                grid.push_cell(CellType::Quad, &ids);
            }
        }
    } else {
        // 3D: generate hexahedra
        let mut pt_map: std::collections::HashMap<[i64; 3], usize> =
            std::collections::HashMap::new();
        for k in 0..gs[2] {
            for j in 0..gs[1] {
                for i in 0..gs[0] {
                    let x0 = origin[0] + i as f64 * spacing[0];
                    let y0 = origin[1] + j as f64 * spacing[1];
                    let z0 = origin[2] + k as f64 * spacing[2];
                    let x1 = x0 + spacing[0];
                    let y1 = y0 + spacing[1];
                    let z1 = z0 + spacing[2];

                    let corners = [
                        [x0, y0, z0],
                        [x1, y0, z0],
                        [x1, y1, z0],
                        [x0, y1, z0],
                        [x0, y0, z1],
                        [x1, y0, z1],
                        [x1, y1, z1],
                        [x0, y1, z1],
                    ];
                    let mut ids = Vec::new();
                    for c in &corners {
                        let key = [
                            (c[0] * 1e6) as i64,
                            (c[1] * 1e6) as i64,
                            (c[2] * 1e6) as i64,
                        ];
                        let idx = *pt_map.entry(key).or_insert_with(|| {
                            let idx = grid.points.len();
                            grid.points.push(*c);
                            idx
                        });
                        ids.push(idx as i64);
                    }
                    grid.push_cell(CellType::Hexahedron, &ids);
                }
            }
        }
    }

    grid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn htg_2d_to_ug() {
        let htg = HyperTreeGrid::new([3, 3, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let ug = hyper_tree_grid_to_unstructured_grid(&htg);
        assert_eq!(ug.cells().num_cells(), 4);
        assert_eq!(ug.cell_types()[0], CellType::Quad);
    }

    #[test]
    fn htg_3d_to_ug() {
        let htg = HyperTreeGrid::new([2, 2, 2], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let ug = hyper_tree_grid_to_unstructured_grid(&htg);
        assert_eq!(ug.cells().num_cells(), 1);
        assert_eq!(ug.cell_types()[0], CellType::Hexahedron);
    }

    #[test]
    fn single_cell_2d() {
        // nk=1 → dimension=2 → quad
        let htg = HyperTreeGrid::new([1, 1, 1], [0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);
        let ug = hyper_tree_grid_to_unstructured_grid(&htg);
        assert_eq!(ug.cells().num_cells(), 1);
        assert_eq!(ug.cell_types()[0], CellType::Quad);
        assert_eq!(ug.points.len(), 4);
    }

    #[test]
    fn single_cell_3d() {
        let htg = HyperTreeGrid::new([1, 1, 2], [0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);
        let ug = hyper_tree_grid_to_unstructured_grid(&htg);
        assert_eq!(ug.cells().num_cells(), 1); // singleton axes keep one root cell
        assert_eq!(ug.cell_types()[0], CellType::Hexahedron);
    }

    #[test]
    fn empty_grid_uses_full_grid_bounds() {
        let htg = HyperTreeGrid::new([3, 2, 1], [10.0, 20.0, 0.0], [2.0, 3.0, 1.0]);
        assert!(htg.bounds().is_empty());
        let ug = hyper_tree_grid_to_unstructured_grid(&htg);
        assert_eq!(ug.cells().num_cells(), 2);
        assert_eq!(ug.points.get(0), [10.0, 20.0, 0.0]);
    }
}
