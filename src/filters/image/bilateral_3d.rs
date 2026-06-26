use crate::data::{AnyDataArray, DataArray, ImageData};

/// 3D bilateral filtering of an ImageData scalar field.
///
/// Each voxel is replaced by a weighted average of neighboring voxels. The
/// weight is the product of a spatial Gaussian and a range Gaussian, preserving
/// sharp intensity transitions while smoothing within similar regions.
pub fn image_bilateral_3d(
    input: &ImageData,
    scalars: &str,
    sigma_spatial: f64,
    sigma_range: f64,
    radius: usize,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0];
    let ny = dims[1];
    let nz = dims[2];
    let n = nx * ny * nz;
    if n == 0 || arr.num_tuples() != n || sigma_spatial <= 0.0 || sigma_range <= 0.0 {
        return input.clone();
    }

    let r = radius as i64;
    let spacing = input.spacing();
    let inv_2ss = 1.0 / (2.0 * sigma_spatial * sigma_spatial);
    let inv_2sr = 1.0 / (2.0 * sigma_range * sigma_range);

    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for (i, value) in values.iter_mut().enumerate() {
        arr.tuple_as_f64(i, &mut buf);
        *value = buf[0];
    }

    let mut result = vec![0.0f64; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let idx = k * ny * nx + j * nx + i;
                let center_value = values[idx];
                let mut sum = 0.0;
                let mut weight_sum = 0.0;

                for dz in -r..=r {
                    let kk = k as i64 + dz;
                    if kk < 0 || kk >= nz as i64 {
                        continue;
                    }
                    for dy in -r..=r {
                        let jj = j as i64 + dy;
                        if jj < 0 || jj >= ny as i64 {
                            continue;
                        }
                        for dx in -r..=r {
                            let ii = i as i64 + dx;
                            if ii < 0 || ii >= nx as i64 {
                                continue;
                            }

                            let neighbor_idx =
                                kk as usize * ny * nx + jj as usize * nx + ii as usize;
                            let neighbor_value = values[neighbor_idx];
                            let sx = dx as f64 * spacing[0];
                            let sy = dy as f64 * spacing[1];
                            let sz = dz as f64 * spacing[2];
                            let spatial_distance2 = sx * sx + sy * sy + sz * sz;
                            let range_distance = neighbor_value - center_value;
                            let range_distance2 = range_distance * range_distance;
                            let weight =
                                (-spatial_distance2 * inv_2ss - range_distance2 * inv_2sr).exp();

                            sum += weight * neighbor_value;
                            weight_sum += weight;
                        }
                    }
                }

                result[idx] = if weight_sum > 0.0 {
                    sum / weight_sum
                } else {
                    center_value
                };
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Bilateral3D",
            result,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_field_is_unchanged() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![7.0; 27],
                1,
            )));

        let result = image_bilateral_3d(&img, "v", 1.0, 1.0, 1);
        let arr = result.point_data().get_array("Bilateral3D").unwrap();
        let mut buf = [0.0f64];
        for i in 0..27 {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - 7.0).abs() < 1e-10);
        }
    }

    #[test]
    fn preserves_step_edge() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.0, 0.0, 100.0, 100.0],
                1,
            )));

        let result = image_bilateral_3d(&img, "v", 1.0, 5.0, 2);
        let arr = result.point_data().get_array("Bilateral3D").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert!(buf[0] < 10.0);
        arr.tuple_as_f64(4, &mut buf);
        assert!(buf[0] > 90.0);
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let result = image_bilateral_3d(&img, "missing", 1.0, 1.0, 1);
        assert!(result.point_data().get_array("Bilateral3D").is_none());
    }
}
