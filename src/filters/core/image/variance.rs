use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute local variance of an ImageData scalar field.
///
/// For each voxel, computes the variance of values in a cubic
/// neighborhood of the given radius. Adds a "Variance" array.
/// Useful for texture analysis and edge detection.
pub fn image_variance(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let r = radius.max(1) as i64;

    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let mut variance = vec![0.0f64; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut sum = 0.0;
                let mut sum_sq = 0.0;
                let mut count = 0usize;

                for dk in -r..=r {
                    let kk = (k as i64 + dk).clamp(0, nz as i64 - 1) as usize;
                    for dj in -r..=r {
                        let jj = (j as i64 + dj).clamp(0, ny as i64 - 1) as usize;
                        for di in -r..=r {
                            let ii = (i as i64 + di).clamp(0, nx as i64 - 1) as usize;
                            let v = values[kk * ny * nx + jj * nx + ii];
                            sum += v;
                            sum_sq += v * v;
                            count += 1;
                        }
                    }
                }

                let mean = sum / count as f64;
                variance[k * ny * nx + j * nx + i] = (sum_sq / count as f64 - mean * mean).max(0.0);
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

    #[test]
    fn uniform_zero_variance() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![5.0; 9], 1)));

        let result = image_variance(&img, "v", 1);
        let arr = result.point_data().get_array("Variance").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0].abs() < 1e-10);
        }
    }

    #[test]
    fn high_contrast_high_variance() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 100.0, 0.0],
                1,
            )));

        let result = image_variance(&img, "v", 1);
        let arr = result.point_data().get_array("Variance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert!(buf[0] > 100.0); // high variance around the spike
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = image_variance(&img, "nope", 1);
        assert!(result.point_data().get_array("Variance").is_none());
    }
}
