use crate::data::{AnyDataArray, DataArray, ImageData};

/// Bilateral filtering of an ImageData scalar field.
///
/// Edge-preserving smoothing: combines spatial proximity and intensity
/// similarity. Points with similar values are averaged more strongly.
///
/// `sigma_spatial`: controls spatial falloff (in voxels).
/// `sigma_range`: controls intensity similarity falloff.
pub fn image_bilateral(
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
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    if n == 0 || arr.num_tuples() != n || sigma_spatial <= 0.0 || sigma_range <= 0.0 {
        return input.clone();
    }
    let r = radius.max(1) as i64;
    let inv_2ss = 1.0 / (2.0 * sigma_spatial * sigma_spatial);
    let inv_2sr = 1.0 / (2.0 * sigma_range * sigma_range);

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
                let center_val = values[k * ny * nx + j * nx + i];
                let mut sum_w = 0.0;
                let mut sum_v = 0.0;

                for dk in -r..=r {
                    let kk = (k as i64 + dk).clamp(0, nz as i64 - 1) as usize;
                    for dj in -r..=r {
                        let jj = (j as i64 + dj).clamp(0, ny as i64 - 1) as usize;
                        for di in -r..=r {
                            let ii = (i as i64 + di).clamp(0, nx as i64 - 1) as usize;

                            let d_spatial = (di * di + dj * dj + dk * dk) as f64;
                            let v = values[kk * ny * nx + jj * nx + ii];
                            let d_range = (v - center_val) * (v - center_val);

                            let w = (-d_spatial * inv_2ss - d_range * inv_2sr).exp();
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
    let mut new_attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                result.clone(),
                1,
            )));
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = new_attrs;
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_edges() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        // Step edge: [0, 0, 100, 100, 100]
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.0, 100.0, 100.0, 100.0],
                1,
            )));

        let result = image_bilateral(&img, "v", 1.0, 10.0, 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        // Left side should stay near 0
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 30.0);
        // Right side should stay near 100
        arr.tuple_as_f64(4, &mut buf);
        assert!(buf[0] > 70.0);
    }

    #[test]
    fn smooths_noise() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0, 10.0, 50.0, 10.0, 10.0],
                1,
            )));

        let result = image_bilateral(&img, "v", 1.0, 100.0, 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        // Large sigma_range -> behaves like Gaussian, spike reduced
        assert!(buf[0] < 50.0);
    }

    #[test]
    fn uniform_unchanged() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![5.0; 9], 1)));

        let result = image_bilateral(&img, "v", 1.0, 1.0, 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - 5.0).abs() < 1e-10);
        }
    }
}
