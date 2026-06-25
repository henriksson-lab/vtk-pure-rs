use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute local variance of an ImageData scalar field in an NxNxN window.
///
/// For each voxel, computes the variance of values in a cubic neighborhood
/// of the given `radius` (the window is `(2*radius+1)^3`). A "Variance"
/// array is added to the output point data.
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
    let r: i64 = radius.max(1) as i64;

    // Extract scalar values
    let mut values: Vec<f64> = vec![0.0; total];
    let mut buf: [f64; 1] = [0.0];
    for i in 0..total {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let mut variance: Vec<f64> = vec![0.0; total];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut sum: f64 = 0.0;
                let mut sum_sq: f64 = 0.0;
                let mut count: usize = 0;

                for dk in -r..=r {
                    let kk: usize = (k as i64 + dk).clamp(0, nz as i64 - 1) as usize;
                    for dj in -r..=r {
                        let jj: usize = (j as i64 + dj).clamp(0, ny as i64 - 1) as usize;
                        for di in -r..=r {
                            let ii: usize = (i as i64 + di).clamp(0, nx as i64 - 1) as usize;
                            let v: f64 = values[kk * ny * nx + jj * nx + ii];
                            sum += v;
                            sum_sq += v * v;
                            count += 1;
                        }
                    }
                }

                let mean: f64 = sum / count as f64;
                let var: f64 = (sum_sq / count as f64 - mean * mean).max(0.0);
                variance[k * ny * nx + j * nx + i] = var;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Variance", variance, 1,
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
