use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute VTK-style local variance of an ImageData scalar field in a window.
///
/// For each voxel, computes the mean squared difference between each
/// in-bounds neighbor in an ellipsoidal footprint and the center voxel.
/// A "Variance" array is added to the output point data.
///
/// If the named scalar array is not found, returns a clone of the input.
pub fn variance_filter(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let total: usize = nx * ny * nz;
    let num_comps: usize = arr.num_components();
    if total == 0 || arr.num_tuples() < total {
        return input.clone();
    }
    let r: i64 = radius as i64;
    let kernel_radius: f64 = radius as f64 + 0.5;

    let mut values: Vec<f64> = vec![0.0; total * num_comps];
    let mut buf: Vec<f64> = vec![0.0; num_comps];
    for i in 0..total {
        arr.tuple_as_f64(i, &mut buf);
        values[i * num_comps..(i + 1) * num_comps].copy_from_slice(&buf);
    }

    let mut variance: Vec<f64> = vec![0.0; total * num_comps];

    for comp in 0..num_comps {
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let center_idx: usize = (k * ny * nx + j * nx + i) * num_comps + comp;
                    let center: f64 = values[center_idx];
                    let mut sum: f64 = 0.0;
                    let mut count: usize = 0;

                    for dk in -r..=r {
                        let kk: i64 = k as i64 + dk;
                        if kk < 0 || kk >= nz as i64 {
                            continue;
                        }
                        for dj in -r..=r {
                            let jj: i64 = j as i64 + dj;
                            if jj < 0 || jj >= ny as i64 {
                                continue;
                            }
                            for di in -r..=r {
                                let mask: bool = if kernel_radius > 0.0 {
                                    let s0 = di as f64 / kernel_radius;
                                    let s1 = dj as f64 / kernel_radius;
                                    let s2 = dk as f64 / kernel_radius;
                                    s0 * s0 + s1 * s1 + s2 * s2 <= 1.0
                                } else {
                                    di == 0 && dj == 0 && dk == 0
                                };
                                if !mask {
                                    continue;
                                }
                                let ii: i64 = i as i64 + di;
                                if ii < 0 || ii >= nx as i64 {
                                    continue;
                                }
                                let idx: usize =
                                    (kk as usize * ny * nx + jj as usize * nx + ii as usize)
                                        * num_comps
                                        + comp;
                                let diff: f64 = values[idx] - center;
                                sum += diff * diff;
                                count += 1;
                            }
                        }
                    }

                    variance[center_idx] = if count > 0 { sum / count as f64 } else { 0.0 };
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Variance", variance, num_comps,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_uniform_image() -> ImageData {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        let n: usize = 27;
        let values: Vec<f64> = vec![5.0; n];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("Scalars", values, 1)));
        img
    }

    fn make_gradient_image() -> ImageData {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let n: usize = 25;
        let mut values: Vec<f64> = Vec::with_capacity(n);
        for i in 0..n {
            values.push(i as f64);
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("Scalars", values, 1)));
        img
    }

    #[test]
    fn uniform_has_zero_variance() {
        let img = make_uniform_image();
        let result = variance_filter(&img, "Scalars", 1);
        let var_arr = result.point_data().get_array("Variance").unwrap();
        let mut buf: [f64; 1] = [0.0];
        for i in 0..27 {
            var_arr.tuple_as_f64(i, &mut buf);
            assert!(
                buf[0].abs() < 1e-10,
                "variance at {} should be 0, got {}",
                i,
                buf[0]
            );
        }
    }

    #[test]
    fn gradient_has_positive_variance() {
        let img = make_gradient_image();
        let result = variance_filter(&img, "Scalars", 1);
        let var_arr = result.point_data().get_array("Variance").unwrap();
        let mut buf: [f64; 1] = [0.0];
        // Center voxel (2,2,0) index=12 should have nonzero variance
        var_arr.tuple_as_f64(12, &mut buf);
        assert!(
            buf[0] > 0.0,
            "center variance should be positive, got {}",
            buf[0]
        );
    }

    #[test]
    fn missing_scalars_returns_clone() {
        let img = ImageData::with_dimensions(2, 2, 2);
        let result = variance_filter(&img, "NonExistent", 1);
        assert_eq!(result.dimensions(), [2, 2, 2]);
    }
}
