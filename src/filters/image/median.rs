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
    let r = radius.max(1) as i64;

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
                let mut neighborhood = Vec::new();
                for dk in -r..=r {
                    let kk = (k as i64 + dk).clamp(0, nz as i64 - 1) as usize;
                    for dj in -r..=r {
                        let jj = (j as i64 + dj).clamp(0, ny as i64 - 1) as usize;
                        for di in -r..=r {
                            let ii = (i as i64 + di).clamp(0, nx as i64 - 1) as usize;
                            neighborhood.push(values[kk * ny * nx + jj * nx + ii]);
                        }
                    }
                }
                neighborhood.sort_by(|a, b| a.partial_cmp(b).unwrap());
                result[k * ny * nx + j * nx + i] = neighborhood[neighborhood.len() / 2];
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
