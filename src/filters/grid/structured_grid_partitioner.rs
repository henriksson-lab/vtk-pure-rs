//! Partition structured grids into sub-grids.

use crate::data::{AnyDataArray, DataArray, StructuredGrid};

/// Partition a StructuredGrid into N sub-grids along the longest dimension.
///
/// Returns the grid with a "PartitionId" cell data array.
pub fn partition_structured_grid(grid: &StructuredGrid, n_partitions: usize) -> StructuredGrid {
    let dims = grid.dimensions();
    if dims[0] == 0 || dims[1] == 0 || dims[2] == 0 || n_partitions == 0 {
        return grid.clone();
    }

    // Cell dimensions
    let cdims = [
        dims[0].saturating_sub(1).max(1),
        dims[1].saturating_sub(1).max(1),
        dims[2].saturating_sub(1).max(1),
    ];
    let n_cells = cdims[0] * cdims[1] * cdims[2];

    // Partition along longest cell dimension
    let longest = if cdims[0] >= cdims[1] && cdims[0] >= cdims[2] {
        0
    } else if cdims[1] >= cdims[2] {
        1
    } else {
        2
    };

    let mut partition_ids = Vec::with_capacity(n_cells);
    for iz in 0..cdims[2] {
        for iy in 0..cdims[1] {
            for ix in 0..cdims[0] {
                let idx = [ix, iy, iz][longest];
                let partition = (idx * n_partitions / cdims[longest]).min(n_partitions - 1);
                partition_ids.push(partition as f64);
            }
        }
    }

    let mut result = grid.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "PartitionId",
            partition_ids,
            1,
        )));
    result
}

/// Compute load balance quality (ratio of min to max cells per partition).
pub fn partition_balance(grid: &StructuredGrid) -> f64 {
    let arr = match grid.cell_data().get_array("PartitionId") {
        Some(a) => a,
        None => return 1.0,
    };

    let mut counts: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    let mut buf = [0.0f64];
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        *counts.entry(buf[0] as i64).or_insert(0) += 1;
    }

    if counts.is_empty() {
        return 1.0;
    }
    let min = *counts.values().min().unwrap() as f64;
    let max = *counts.values().max().unwrap() as f64;
    if max < 1e-15 {
        1.0
    } else {
        min / max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid() -> StructuredGrid {
        StructuredGrid::from_dimensions_and_points(
            [4, 3, 2],
            (0..24)
                .map(|i| {
                    let ix = i % 4;
                    let iy = (i / 4) % 3;
                    let iz = i / 12;
                    [ix as f64, iy as f64, iz as f64]
                })
                .collect(),
        )
    }

    #[test]
    fn partition_2() {
        let grid = make_grid();
        let result = partition_structured_grid(&grid, 2);
        assert!(result.cell_data().get_array("PartitionId").is_some());

        let arr = result.cell_data().get_array("PartitionId").unwrap();
        let mut ids = std::collections::HashSet::new();
        let mut buf = [0.0f64];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            ids.insert(buf[0] as i64);
        }
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn balance() {
        let grid = make_grid();
        let result = partition_structured_grid(&grid, 3);
        let b = partition_balance(&result);
        assert!(b > 0.0 && b <= 1.0);
    }

    #[test]
    fn single_partition() {
        let grid = make_grid();
        let result = partition_structured_grid(&grid, 1);
        let arr = result.cell_data().get_array("PartitionId").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
