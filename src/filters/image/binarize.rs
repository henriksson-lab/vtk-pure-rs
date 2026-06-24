use crate::data::{AnyDataArray, DataArray, ImageData};

/// Binarize an ImageData using adaptive (local mean) thresholding.
///
/// For each voxel, the threshold is the local mean in a neighborhood
/// of `radius`, offset by `bias`. Produces a "Binary" array with 0/1 values.
pub fn image_adaptive_threshold(
    input: &ImageData,
    scalars: &str,
    radius: usize,
    bias: f64,
) -> ImageData {
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
    let mut values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut binary = vec![0.0f64; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut sum = 0.0;
                let mut count = 0;
                for dk in -r..=r {
                    for dj in -r..=r {
                        for di in -r..=r {
                            let ii = (i as i64 + di).clamp(0, nx as i64 - 1) as usize;
                            let jj = (j as i64 + dj).clamp(0, ny as i64 - 1) as usize;
                            let kk = (k as i64 + dk).clamp(0, nz as i64 - 1) as usize;
                            sum += values[kk * ny * nx + jj * nx + ii];
                            count += 1;
                        }
                    }
                }
                let local_mean = sum / count as f64;
                let idx = k * ny * nx + j * nx + i;
                binary[idx] = if values[idx] > local_mean + bias {
                    1.0
                } else {
                    0.0
                };
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Binary", binary, 1)));
    img
}

/// Simple global binarization: values >= threshold become 1.0, else 0.0.
pub fn image_binarize(input: &ImageData, scalars: &str, threshold: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] >= threshold {
                1.0
            } else {
                0.0
            }
        })
        .collect();

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Binary", values, 1)));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_threshold() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0, 10.0, 50.0, 10.0, 10.0],
                1,
            )));

        let result = image_adaptive_threshold(&img, "v", 1, 0.0);
        let arr = result.point_data().get_array("Binary").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 1.0); // spike above local mean
    }

    #[test]
    fn global_binarize() {
        let mut img = ImageData::with_dimensions(4, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.1, 0.5, 0.9, 0.3],
                1,
            )));

        let result = image_binarize(&img, "v", 0.5);
        let arr = result.point_data().get_array("Binary").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = image_adaptive_threshold(&img, "nope", 1, 0.0);
        assert!(r.point_data().get_array("Binary").is_none());
    }
}
