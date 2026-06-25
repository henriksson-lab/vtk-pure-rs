use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute local range (max - min) in a neighborhood.
///
/// Adds "LocalRange" scalar. High values = edges, low values = flat regions.
pub fn image_local_range(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
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
                let mut lo = f64::MAX;
                let mut hi = f64::MIN;
                for dk in -r..=r {
                    for dj in -r..=r {
                        for di in -r..=r {
                            let ii = (i as i64 + di).clamp(0, nx as i64 - 1) as usize;
                            let jj = (j as i64 + dj).clamp(0, ny as i64 - 1) as usize;
                            let kk = (k as i64 + dk).clamp(0, nz as i64 - 1) as usize;
                            let v = values[kk * ny * nx + jj * nx + ii];
                            lo = lo.min(v);
                            hi = hi.max(v);
                        }
                    }
                }
                result[k * ny * nx + j * nx + i] = hi - lo;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "LocalRange",
            result,
            1,
        )));
    img
}

/// Compute local standard deviation in a neighborhood.
pub fn image_local_std(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
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
                result[k * ny * nx + j * nx + i] =
                    (sum2 / cnt as f64 - mean * mean).max(0.0).sqrt();
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "LocalStd", result, 1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_high_range() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.0, 100.0, 100.0, 100.0],
                1,
            )));

        let result = image_local_range(&img, "v", 1);
        let arr = result.point_data().get_array("LocalRange").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 100.0); // edge
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 0.0); // interior
    }

    #[test]
    fn uniform_zero_std() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![5.0; 9], 1)));

        let result = image_local_std(&img, "v", 1);
        let arr = result.point_data().get_array("LocalStd").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0] < 1e-10);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let r = image_local_range(&img, "nope", 1);
        assert!(r.point_data().get_array("LocalRange").is_none());
    }
}
