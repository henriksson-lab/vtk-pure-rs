use crate::data::{AnyDataArray, DataArray, ImageData};

/// Bilateral filter on ImageData: edge-preserving smoothing using both spatial
/// and intensity distance weights.
///
/// For each voxel, neighboring voxels contribute based on their spatial distance
/// (controlled by `spatial_sigma`) and intensity difference (controlled by
/// `intensity_sigma`). The kernel radius is derived from `spatial_sigma`.
///
/// The result is stored as a "BilateralFiltered" point data array on the
/// returned ImageData.
pub fn bilateral_filter(
    input: &ImageData,
    scalars: &str,
    spatial_sigma: f64,
    intensity_sigma: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n: usize = nx * ny * nz;
    if n == 0 {
        return input.clone();
    }
    let spacing = input.spacing();
    let spatial_sigma = if spatial_sigma.is_finite() && spatial_sigma > 0.0 {
        spatial_sigma
    } else {
        f64::EPSILON
    };
    let intensity_sigma = if intensity_sigma.is_finite() && intensity_sigma > 0.0 {
        intensity_sigma
    } else {
        f64::EPSILON
    };

    // Kernel radius: 2 * sigma in voxels, at least 1
    let radius: i64 = (2.0 * spatial_sigma).ceil().max(1.0) as i64;

    let inv_2s_sq: f64 = 1.0 / (2.0 * spatial_sigma * spatial_sigma);
    let inv_2i_sq: f64 = 1.0 / (2.0 * intensity_sigma * intensity_sigma);

    // Read scalar values
    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let mut result = vec![0.0f64; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let center_val: f64 = values[k * ny * nx + j * nx + i];
                let mut sum_w: f64 = 0.0;
                let mut sum_v: f64 = 0.0;

                for dk in -radius..=radius {
                    let kk = k as i64 + dk;
                    if kk < 0 || kk >= nz as i64 {
                        continue;
                    }
                    let kk = kk as usize;
                    for dj in -radius..=radius {
                        let jj = j as i64 + dj;
                        if jj < 0 || jj >= ny as i64 {
                            continue;
                        }
                        let jj = jj as usize;
                        for di in -radius..=radius {
                            let ii = i as i64 + di;
                            if ii < 0 || ii >= nx as i64 {
                                continue;
                            }
                            let ii = ii as usize;

                            // Spatial distance squared (in world coords)
                            let dx: f64 = di as f64 * spacing[0];
                            let dy: f64 = dj as f64 * spacing[1];
                            let dz: f64 = dk as f64 * spacing[2];
                            let d_spatial: f64 = dx * dx + dy * dy + dz * dz;

                            let v: f64 = values[kk * ny * nx + jj * nx + ii];
                            let d_intensity: f64 = (v - center_val) * (v - center_val);

                            let w: f64 = (-d_spatial * inv_2s_sq - d_intensity * inv_2i_sq).exp();
                            sum_w += w;
                            sum_v += w * v;
                        }
                    }
                }

                result[k * ny * nx + j * nx + i] = if sum_w > 1e-15 {
                    sum_v / sum_w
                } else {
                    center_val
                };
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BilateralFiltered",
            result,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_field_unchanged() {
        let mut img = ImageData::with_dimensions(4, 4, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "vals",
                vec![5.0; 16],
                1,
            )));

        let result = bilateral_filter(&img, "vals", 1.0, 1.0);
        let arr = result.point_data().get_array("BilateralFiltered").unwrap();
        let mut buf = [0.0f64];
        for i in 0..16 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(
                (buf[0] - 5.0).abs() < 1e-10,
                "uniform field should be preserved"
            );
        }
    }

    #[test]
    fn preserves_step_edge() {
        let mut img = ImageData::with_dimensions(6, 1, 1);
        // Step edge: low on left, high on right
        let vals: Vec<f64> = vec![0.0, 0.0, 0.0, 100.0, 100.0, 100.0];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("vals", vals, 1)));

        let result = bilateral_filter(&img, "vals", 1.0, 5.0);
        let arr = result.point_data().get_array("BilateralFiltered").unwrap();
        let mut buf = [0.0f64];
        // Far left should stay near 0
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 20.0, "left side should remain low: {}", buf[0]);
        // Far right should stay near 100
        arr.tuple_as_f64(5, &mut buf);
        assert!(buf[0] > 80.0, "right side should remain high: {}", buf[0]);
    }

    #[test]
    fn missing_scalars_returns_clone() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = bilateral_filter(&img, "nonexistent", 1.0, 1.0);
        assert_eq!(result.dimensions(), [3, 3, 1]);
    }
}
