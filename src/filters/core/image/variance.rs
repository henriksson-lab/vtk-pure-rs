use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute VTK-style local variance of an ImageData scalar field.
///
/// This follows `vtkImageVariance3D`: for each voxel, average the squared
/// difference between each in-bounds ellipsoidal-kernel neighbor and the
/// center voxel. The value is not the statistical variance of the neighborhood.
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
    if n == 0 {
        return input.clone();
    }
    let r = radius as i64;
    let kernel_size = 2.0 * radius as f64 + 1.0;
    let kernel_radius = kernel_size * 0.5;

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
                let center = values[k * ny * nx + j * nx + i];
                let mut sum = 0.0;
                let mut count = 0usize;

                for dk in -r..=r {
                    let kk = k as i64 + dk;
                    if kk < 0 || kk >= nz as i64 {
                        continue;
                    }
                    for dj in -r..=r {
                        let jj = j as i64 + dj;
                        if jj < 0 || jj >= ny as i64 {
                            continue;
                        }
                        for di in -r..=r {
                            let ii = i as i64 + di;
                            if ii < 0 || ii >= nx as i64 {
                                continue;
                            }
                            if radius > 0 {
                                let x = di as f64 / kernel_radius;
                                let y = dj as f64 / kernel_radius;
                                let z = dk as f64 / kernel_radius;
                                if x * x + y * y + z * z > 1.0 {
                                    continue;
                                }
                            }
                            let v = values[kk as usize * ny * nx + jj as usize * nx + ii as usize];
                            let diff = v - center;
                            sum += diff * diff;
                            count += 1;
                        }
                    }
                }

                variance[k * ny * nx + j * nx + i] =
                    if count > 0 { sum / count as f64 } else { 0.0 };
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
    fn center_difference_not_statistical_variance() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 10.0, 0.0],
                1,
            )));

        let result = image_variance(&img, "v", 1);
        let arr = result.point_data().get_array("Variance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - (200.0 / 3.0)).abs() < 1e-10);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = image_variance(&img, "nope", 1);
        assert!(result.point_data().get_array("Variance").is_none());
    }
}
