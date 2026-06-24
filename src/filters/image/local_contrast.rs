use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute local contrast enhancement (CLAHE-like).
///
/// Divides the image into tiles and applies histogram equalization
/// within each tile, then interpolates. Simplified version using
/// local normalization: (value - local_mean) / local_stddev.
pub fn image_local_contrast(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
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

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut result = vec![0.0f64; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut sum = 0.0;
                let mut sum2 = 0.0;
                let mut cnt = 0;
                for dk in -r..=r {
                    for dj in -r..=r {
                        for di in -r..=r {
                            let ii = (i as i64 + di).clamp(0, nx as i64 - 1) as usize;
                            let jj = (j as i64 + dj).clamp(0, ny as i64 - 1) as usize;
                            let kk = (k as i64 + dk).clamp(0, nz as i64 - 1) as usize;
                            let v = values[kk * ny * nx + jj * nx + ii];
                            sum += v;
                            sum2 += v * v;
                            cnt += 1;
                        }
                    }
                }
                let mean = sum / cnt as f64;
                let var = (sum2 / cnt as f64 - mean * mean).max(0.0);
                let std = var.sqrt().max(1e-10);
                result[k * ny * nx + j * nx + i] = (values[k * ny * nx + j * nx + i] - mean) / std;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "LocalContrast",
            result,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enhances_contrast() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![50.0; 25];
        values[12] = 100.0; // bright spot
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_local_contrast(&img, "v", 1);
        let arr = result.point_data().get_array("LocalContrast").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf);
        assert!(buf[0] > 0.0); // above local mean
    }

    #[test]
    fn uniform_zero_contrast() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![5.0; 9], 1)));

        let result = image_local_contrast(&img, "v", 1);
        let arr = result.point_data().get_array("LocalContrast").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0].abs() < 1e-5);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let r = image_local_contrast(&img, "nope", 1);
        assert!(r.point_data().get_array("LocalContrast").is_none());
    }
}
