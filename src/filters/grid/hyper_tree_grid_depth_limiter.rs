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
    let bounds = htg.bounds();

    let new_gs = [
        gs[0].min(max_coarse_cells_per_axis),
        gs[1].min(max_coarse_cells_per_axis),
        if gs[2] > 1 {
            gs[2].min(max_coarse_cells_per_axis)
        } else {
            1
        },
    ];

    let new_spacing = [
        (bounds.x_max - bounds.x_min) / new_gs[0] as f64,
        (bounds.y_max - bounds.y_min) / new_gs[1] as f64,
        if new_gs[2] > 1 {
            (bounds.z_max - bounds.z_min) / new_gs[2] as f64
        } else {
            1.0
        },
    ];

    HyperTreeGrid::new(
        new_gs,
        [bounds.x_min, bounds.y_min, bounds.z_min],
        new_spacing,
    )
}

/// Convert a HyperTreeGrid to a uniform grid at a specified resolution.
///
/// This effectively "flattens" the adaptive resolution to a fixed grid.
pub fn htg_to_uniform(htg: &HyperTreeGrid, resolution: [usize; 3]) -> ImageData {
    let bounds = htg.bounds();
    let spacing = [
        (bounds.x_max - bounds.x_min) / resolution[0] as f64,
        (bounds.y_max - bounds.y_min) / resolution[1] as f64,
        if resolution[2] > 1 {
            (bounds.z_max - bounds.z_min) / resolution[2] as f64
        } else {
            1.0
        },
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
    }

    #[test]
    fn to_uniform() {
        let htg = HyperTreeGrid::new([4, 4, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let img = htg_to_uniform(&htg, [20, 20, 1]);
        assert_eq!(img.dimensions(), [20, 20, 1]);
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
        assert_eq!(limited.grid_size(), [2, 2, 1]); // unchanged
    }
}
