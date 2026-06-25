use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply median filtering to an ImageData scalar field.
///
/// Replaces each voxel with the median of its cubic neighborhood
/// of the given `radius`. Robust to salt-and-pepper noise.
pub fn image_median(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let r = radius as i64;
    let num_components = arr.num_components();

    let mut values = vec![0.0f64; n * num_components];
    let mut buf = vec![0.0f64; num_components];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i * num_components..(i + 1) * num_components].copy_from_slice(&buf);
    }

    let mut result = vec![0.0f64; n * num_components];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                for c in 0..num_components {
                    let mut neighborhood = Vec::new();
                    for kk in (k as i64 - r).max(0)..=(k as i64 + r).min(nz as i64 - 1) {
                        for jj in (j as i64 - r).max(0)..=(j as i64 + r).min(ny as i64 - 1) {
                            for ii in (i as i64 - r).max(0)..=(i as i64 + r).min(nx as i64 - 1) {
                                let idx = kk as usize * ny * nx + jj as usize * nx + ii as usize;
                                neighborhood.push(values[idx * num_components + c]);
                            }
                        }
                    }
                    neighborhood.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let mid = neighborhood.len() / 2;
                    let median = if neighborhood.len() % 2 == 0 {
                        neighborhood[mid - 1] + (neighborhood[mid] - neighborhood[mid - 1]) / 2.0
                    } else {
                        neighborhood[mid]
                    };
                    result[(k * ny * nx + j * nx + i) * num_components + c] = median;
                }
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
                num_components,
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
    fn removes_noise() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        // Spike noise: [0, 0, 100, 0, 0]
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.0, 100.0, 0.0, 0.0],
                1,
            )));

        let result = image_median(&img, "v", 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.0); // median of [0,0,100] = 0
    }

    #[test]
    fn preserves_uniform() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![7.0; 9], 1)));

        let result = image_median(&img, "v", 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 7.0);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = image_median(&img, "nope", 1);
        assert!(result.point_data().get_array("nope").is_none());
    }
}
