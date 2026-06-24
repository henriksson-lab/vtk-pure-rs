use crate::data::{AnyDataArray, DataArray, ImageData};

/// Morphological dilation of a binary ImageData field.
///
/// For each voxel, sets the output to the maximum value in a cubic
/// neighborhood of the given `radius`. Works on the named scalar array.
pub fn image_dilate(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    morphological_op(input, scalars, radius, true)
}

/// Morphological erosion of a binary ImageData field.
///
/// For each voxel, sets the output to the minimum value in a cubic
/// neighborhood of the given `radius`.
pub fn image_erode(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    morphological_op(input, scalars, radius, false)
}

fn morphological_op(input: &ImageData, scalars: &str, radius: usize, dilate: bool) -> ImageData {
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
                let mut best = if dilate { f64::MIN } else { f64::MAX };

                for dk in -r..=r {
                    let kk = (k as i64 + dk).clamp(0, nz as i64 - 1) as usize;
                    for dj in -r..=r {
                        let jj = (j as i64 + dj).clamp(0, ny as i64 - 1) as usize;
                        for di in -r..=r {
                            let ii = (i as i64 + di).clamp(0, nx as i64 - 1) as usize;
                            let v = values[kk * ny * nx + jj * nx + ii];
                            if dilate {
                                best = best.max(v);
                            } else {
                                best = best.min(v);
                            }
                        }
                    }
                }

                result[k * ny * nx + j * nx + i] = best;
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

    fn make_binary_image() -> ImageData {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![0.0f64; 25];
        values[12] = 1.0; // center pixel
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));
        img
    }

    #[test]
    fn dilate_spreads() {
        let img = make_binary_image();
        let result = image_dilate(&img, "mask", 1);
        let arr = result.point_data().get_array("mask").unwrap();
        let mut buf = [0.0f64];
        // Center should still be 1
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 1.0);
        // Neighbors should now be 1
        arr.tuple_as_f64(11, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(7, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn erode_shrinks() {
        // Start with a 3x3 block of 1s
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![0.0f64; 25];
        for j in 1..4 {
            for i in 1..4 {
                values[j * 5 + i] = 1.0;
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));

        let result = image_erode(&img, "mask", 1);
        let arr = result.point_data().get_array("mask").unwrap();
        let mut buf = [0.0f64];
        // Only center should remain
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 1.0);
        // Edge of block should be eroded
        arr.tuple_as_f64(6, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn missing_array() {
        let img = make_binary_image();
        let result = image_dilate(&img, "nope", 1);
        assert!(result.point_data().get_array("mask").is_some());
    }
}
