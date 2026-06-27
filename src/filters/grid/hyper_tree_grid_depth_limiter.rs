//! Limit the depth of a HyperTreeGrid.
//!
//! Since the internal tree structure is private, this operates by
//! reporting depth information and creating a new coarser HTG.

use crate::data::{HyperTreeGrid, ImageData};

/// Get the maximum tree depth across all trees.
pub fn htg_max_depth(htg: &HyperTreeGrid) -> usize {
    htg.max_depth()
}

/// Create a coarser version of the HyperTreeGrid by using a larger spacing.
///
/// Effectively limits resolution by creating a new grid with fewer coarse cells.
pub fn limit_resolution(htg: &HyperTreeGrid, max_coarse_cells_per_axis: usize) -> HyperTreeGrid {
    let gs = htg.grid_size();
    let dims = htg.dimensions();
    let bounds = htg.grid_bounds();
    let max_cells = max_coarse_cells_per_axis.max(1);

    let new_cell_dims = [
        gs[0].min(max_cells),
        gs[1].min(max_cells),
        gs[2].min(max_cells),
    ];
    if new_cell_dims == gs {
        return htg.clone();
    }
    let new_dims = [
        if dims[0] > 1 { new_cell_dims[0] + 1 } else { 1 },
        if dims[1] > 1 { new_cell_dims[1] + 1 } else { 1 },
        if dims[2] > 1 { new_cell_dims[2] + 1 } else { 1 },
    ];

    let new_spacing = [
        (bounds.x_max - bounds.x_min) / new_cell_dims[0] as f64,
        (bounds.y_max - bounds.y_min) / new_cell_dims[1] as f64,
        if dims[2] > 1 {
            (bounds.z_max - bounds.z_min) / new_cell_dims[2] as f64
        } else {
            1.0
        },
    ];

    let mut limited = HyperTreeGrid::new(
        new_dims,
        [bounds.x_min, bounds.y_min, bounds.z_min],
        new_spacing,
    );
    limited.set_branch_factor(htg.branch_factor());
    limited
}

/// Convert a HyperTreeGrid to a uniform grid at a specified resolution.
///
/// This effectively "flattens" the adaptive resolution to a fixed grid.
pub fn htg_to_uniform(htg: &HyperTreeGrid, resolution: [usize; 3]) -> ImageData {
    let bounds = htg.grid_bounds();
    let axis_spacing = |min: f64, max: f64, n: usize| {
        if n > 1 {
            (max - min) / (n - 1) as f64
        } else {
            1.0
        }
    };
    let spacing = [
        axis_spacing(bounds.x_min, bounds.x_max, resolution[0]),
        axis_spacing(bounds.y_min, bounds.y_max, resolution[1]),
        axis_spacing(bounds.z_min, bounds.z_max, resolution[2]),
    ];

    ImageData::with_dimensions(resolution[0], resolution[1], resolution[2])
        .with_spacing(spacing)
        .with_origin([bounds.x_min, bounds.y_min, bounds.z_min])
}

/// Get summary statistics about the HyperTreeGrid depth.
pub fn htg_depth_stats(htg: &HyperTreeGrid) -> (usize, usize, usize) {
    // (num_trees, num_cells, max_depth)
    (htg.num_trees(), htg.num_cells(), htg.max_depth())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_res() {
        let htg = HyperTreeGrid::new([10, 10, 1], [0.0, 0.0, 0.0], [0.1, 0.1, 1.0]);
        let limited = limit_resolution(&htg, 4);
        assert_eq!(limited.grid_size(), [4, 4, 1]);
        assert_eq!(limited.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn to_uniform() {
        let htg = HyperTreeGrid::new([4, 4, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let img = htg_to_uniform(&htg, [20, 20, 1]);
        assert_eq!(img.dimensions(), [20, 20, 1]);
        assert!((img.spacing()[0] - 3.0 / 19.0).abs() < 1e-12);
    }

    #[test]
    fn depth_stats() {
        let mut htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        htg.init_tree(0, 0, 0);
        htg.subdivide(0, 0, 0, 0);
        let (trees, cells, depth) = htg_depth_stats(&htg);
        assert_eq!(trees, 1);
        assert!(cells > 0);
        assert_eq!(depth, 1);
    }

    #[test]
    fn already_small() {
        let htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let limited = limit_resolution(&htg, 10);
        assert_eq!(limited.grid_size(), [1, 1, 1]); // unchanged
    }

    #[test]
    fn limit_empty_grid_uses_grid_bounds() {
        let htg = HyperTreeGrid::new([4, 3, 1], [2.0, 3.0, 0.0], [0.5, 2.0, 1.0]);
        assert!(htg.bounds().is_empty());
        let limited = limit_resolution(&htg, 2);
        let bounds = limited.grid_bounds();
        assert_eq!([bounds.x_min, bounds.y_min], [2.0, 3.0]);
        assert_eq!(limited.grid_size(), [2, 2, 1]);
    }
}
