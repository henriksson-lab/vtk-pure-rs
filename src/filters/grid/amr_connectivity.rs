//! AMR-aware connectivity analysis for HyperTreeGrid.
//!
//! Identifies connected regions in the coarse grid based on cell data
//! thresholds, respecting the AMR grid structure.

use crate::data::{AnyDataArray, DataArray, HyperTreeGrid};

/// Label connected coarse cells that exceed a threshold.
///
/// Uses flood-fill on the coarse grid adjacency. Adds a "RegionId" cell data array.
pub fn amr_connected_regions(
    htg: &HyperTreeGrid,
    array_name: &str,
    threshold: f64,
) -> HyperTreeGrid {
    let gs = htg.grid_size();
    let n = gs[0] * gs[1] * gs[2];

    let arr = match htg.cell_data().get_array(array_name) {
        Some(a) => a,
        None => return htg.clone(),
    };

    // Read values and determine which cells are "active"
    let mut active = vec![false; n];
    let mut buf = [0.0f64];
    for i in 0..n.min(arr.num_tuples()) {
        arr.tuple_as_f64(i, &mut buf);
        active[i] = buf[0] >= threshold;
    }

    let ci = |i: usize, j: usize, k: usize| -> usize { i + j * gs[0] + k * gs[0] * gs[1] };

    // Flood-fill connected components
    let mut labels = vec![-1i64; n];
    let mut next_label = 0i64;

    for seed in 0..n {
        if !active[seed] || labels[seed] >= 0 {
            continue;
        }

        // BFS from seed
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(seed);
        labels[seed] = next_label;

        while let Some(idx) = queue.pop_front() {
            let k = idx / (gs[0] * gs[1]);
            let rem = idx % (gs[0] * gs[1]);
            let j = rem / gs[0];
            let i = rem % gs[0];

            // Check 6-connected neighbors
            let neighbors = [
                if i > 0 { Some(ci(i - 1, j, k)) } else { None },
                if i + 1 < gs[0] {
                    Some(ci(i + 1, j, k))
                } else {
                    None
                },
                if j > 0 { Some(ci(i, j - 1, k)) } else { None },
                if j + 1 < gs[1] {
                    Some(ci(i, j + 1, k))
                } else {
                    None
                },
                if k > 0 { Some(ci(i, j, k - 1)) } else { None },
                if k + 1 < gs[2] {
                    Some(ci(i, j, k + 1))
                } else {
                    None
                },
            ];

            for ni in neighbors.into_iter().flatten() {
                if active[ni] && labels[ni] < 0 {
                    labels[ni] = next_label;
                    queue.push_back(ni);
                }
            }
        }

        next_label += 1;
    }

    let region_data: Vec<f64> = labels
        .iter()
        .map(|&l| if l >= 0 { l as f64 } else { -1.0 })
        .collect();

    let mut result = htg.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "RegionId",
            region_data,
            1,
        )));
    result
}

/// Count the number of connected regions.
pub fn count_amr_regions(htg: &HyperTreeGrid) -> usize {
    match htg.cell_data().get_array("RegionId") {
        Some(arr) => {
            let mut max_id = -1i64;
            let mut buf = [0.0f64];
            for i in 0..arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                max_id = max_id.max(buf[0] as i64);
            }
            if max_id >= 0 {
                (max_id + 1) as usize
            } else {
                0
            }
        }
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_regions() {
        let mut htg = HyperTreeGrid::new([6, 1, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        // Two groups separated by a gap: [1, 1, 0, 1, 1]
        htg.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![1.0, 1.0, 0.0, 1.0, 1.0],
                1,
            )));
        let result = amr_connected_regions(&htg, "val", 0.5);
        let count = count_amr_regions(&result);
        assert_eq!(count, 2);
    }

    #[test]
    fn single_region() {
        let mut htg = HyperTreeGrid::new([4, 4, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let vals = vec![1.0; 9];
        htg.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", vals, 1)));
        let result = amr_connected_regions(&htg, "val", 0.5);
        assert_eq!(count_amr_regions(&result), 1);
    }

    #[test]
    fn no_active_cells() {
        let mut htg = HyperTreeGrid::new([3, 3, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        htg.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0; 4],
                1,
            )));
        let result = amr_connected_regions(&htg, "val", 0.5);
        assert_eq!(count_amr_regions(&result), 0);
    }
}
