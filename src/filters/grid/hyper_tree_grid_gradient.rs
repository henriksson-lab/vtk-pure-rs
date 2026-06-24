//! Gradient computation on HyperTreeGrid coarse cell data.

use crate::data::{AnyDataArray, DataArray, HyperTreeGrid};

/// Compute gradient of a scalar field on HyperTreeGrid coarse cells.
///
/// Uses central differences between adjacent coarse cells.
/// Adds "GradientX", "GradientY", "GradientZ", "GradientMagnitude" arrays.
pub fn hyper_tree_grid_gradient(htg: &HyperTreeGrid, array_name: &str) -> HyperTreeGrid {
    let gs = htg.grid_size();
    let bounds = htg.bounds();
    let spacing = [
        (bounds.x_max - bounds.x_min) / gs[0] as f64,
        (bounds.y_max - bounds.y_min) / gs[1] as f64,
        if gs[2] > 1 {
            (bounds.z_max - bounds.z_min) / gs[2] as f64
        } else {
            1.0
        },
    ];

    let arr = match htg.cell_data().get_array(array_name) {
        Some(a) => a,
        None => return htg.clone(),
    };

    let n = gs[0] * gs[1] * gs[2];
    let ci = |i: usize, j: usize, k: usize| i + j * gs[0] + k * gs[0] * gs[1];

    let mut buf = [0.0f64];
    let _val = |i: usize, j: usize, k: usize| -> f64 {
        let idx = ci(i, j, k);
        if idx < arr.num_tuples() {
            arr.tuple_as_f64(idx, &mut [0.0f64]);
        }
        let mut b = [0.0f64];
        if idx < arr.num_tuples() {
            arr.tuple_as_f64(idx, &mut b);
            b[0]
        } else {
            0.0
        }
    };

    // Read all values first
    let mut values = vec![0.0f64; n];
    for idx in 0..n.min(arr.num_tuples()) {
        arr.tuple_as_f64(idx, &mut buf);
        values[idx] = buf[0];
    }

    let mut gx = vec![0.0f64; n];
    let mut gy = vec![0.0f64; n];
    let mut gz = vec![0.0f64; n];
    let mut gmag = vec![0.0f64; n];

    for k in 0..gs[2] {
        for j in 0..gs[1] {
            for i in 0..gs[0] {
                let idx = ci(i, j, k);

                // Central difference X
                let im = if i > 0 { i - 1 } else { i };
                let ip = if i + 1 < gs[0] { i + 1 } else { i };
                let dx = (ip - im).max(1) as f64 * spacing[0];
                gx[idx] = (values[ci(ip, j, k)] - values[ci(im, j, k)]) / dx;

                // Central difference Y
                let jm = if j > 0 { j - 1 } else { j };
                let jp = if j + 1 < gs[1] { j + 1 } else { j };
                let dy = (jp - jm).max(1) as f64 * spacing[1];
                gy[idx] = (values[ci(i, jp, k)] - values[ci(i, jm, k)]) / dy;

                // Central difference Z
                if gs[2] > 1 {
                    let km = if k > 0 { k - 1 } else { k };
                    let kp = if k + 1 < gs[2] { k + 1 } else { k };
                    let dz = (kp - km).max(1) as f64 * spacing[2];
                    gz[idx] = (values[ci(i, j, kp)] - values[ci(i, j, km)]) / dz;
                }

                gmag[idx] = (gx[idx] * gx[idx] + gy[idx] * gy[idx] + gz[idx] * gz[idx]).sqrt();
            }
        }
    }

    let mut result = htg.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("GradientX", gx, 1)));
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("GradientY", gy, 1)));
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("GradientZ", gz, 1)));
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GradientMagnitude",
            gmag,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_gradient() {
        let mut htg = HyperTreeGrid::new([5, 1, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let vals = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        htg.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("f", vals, 1)));

        let result = hyper_tree_grid_gradient(&htg, "f");
        let gx = result.cell_data().get_array("GradientX").unwrap();
        let mut buf = [0.0f64];
        // Interior cell should have gradient ≈ 1.0
        gx.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn missing_array() {
        let htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let result = hyper_tree_grid_gradient(&htg, "none");
        assert!(result.cell_data().get_array("GradientX").is_none());
    }
}
