use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute VTK-style local variance of an ImageData scalar field.
///
/// For each voxel, computes the mean squared difference between each
/// in-bounds neighbor in an ellipsoidal footprint and the center voxel.
/// Adds a "Variance" array.
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
    let num_comps = arr.num_components();
    if n == 0 || arr.num_tuples() < n {
        return input.clone();
    }
    let r = radius as i64;
    let kernel_radius = radius as f64 + 0.5;

    let mut values = vec![0.0f64; n * num_comps];
    let mut buf = vec![0.0f64; num_comps];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i * num_comps..(i + 1) * num_comps].copy_from_slice(&buf);
    }

    let mut variance = vec![0.0f64; n * num_comps];

    for comp in 0..num_comps {
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let center_idx = (k * ny * nx + j * nx + i) * num_comps + comp;
                    let center = values[center_idx];
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
                                let mask = if kernel_radius > 0.0 {
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
                                let ii = i as i64 + di;
                                if ii < 0 || ii >= nx as i64 {
                                    continue;
                                }
                                let idx = (kk as usize * ny * nx + jj as usize * nx + ii as usize)
                                    * num_comps
                                    + comp;
                                let diff = values[idx] - center;
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
