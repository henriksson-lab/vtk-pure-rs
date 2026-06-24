//! Threshold filter for HyperTreeGrid cell data.

use crate::data::{AnyDataArray, DataArray, HyperTreeGrid};

/// Mark coarse cells that pass a scalar threshold.
///
/// Adds a "ThresholdMask" cell data array (1.0 = pass, 0.0 = fail).
pub fn hyper_tree_grid_threshold(
    htg: &HyperTreeGrid,
    array_name: &str,
    min_val: f64,
    max_val: f64,
) -> HyperTreeGrid {
    let mut result = htg.clone();
    let n_cells = htg.num_cells();

    let arr = match htg.cell_data().get_array(array_name) {
        Some(a) => a,
        None => {
            // No array — mark everything as passing
            let mask = vec![1.0f64; n_cells.max(htg.num_coarse_cells())];
            result
                .cell_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    "ThresholdMask",
                    mask,
                    1,
                )));
            return result;
        }
    };

    let n = arr.num_tuples();
    let mut mask = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        mask.push(if buf[0] >= min_val && buf[0] <= max_val {
            1.0
        } else {
            0.0
        });
    }

    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ThresholdMask",
            mask,
            1,
        )));
    result
}

/// Count cells passing the threshold.
pub fn count_threshold_cells(htg: &HyperTreeGrid) -> usize {
    match htg.cell_data().get_array("ThresholdMask") {
        Some(arr) => {
            let mut count = 0;
            let mut buf = [0.0f64];
            for i in 0..arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                if buf[0] > 0.5 {
                    count += 1;
                }
            }
            count
        }
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_threshold() {
        let mut htg = HyperTreeGrid::new([4, 4, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        // Add cell data
        let vals: Vec<f64> = (0..16).map(|i| i as f64).collect();
        htg.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("density", vals, 1)));

        let result = hyper_tree_grid_threshold(&htg, "density", 5.0, 10.0);
        let count = count_threshold_cells(&result);
        assert_eq!(count, 6); // values 5,6,7,8,9,10
    }

    #[test]
    fn missing_array() {
        let htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let result = hyper_tree_grid_threshold(&htg, "nonexistent", 0.0, 1.0);
        assert!(result.cell_data().get_array("ThresholdMask").is_some());
    }
}
