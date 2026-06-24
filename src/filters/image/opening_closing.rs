use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};

/// Morphological opening (erode then dilate) on a binary ImageData field.
///
/// Opening removes small bright features (noise) while preserving the overall shape.
/// Uses a cubic structuring element of radius 1.
pub fn morphological_opening(input: &ImageData, scalars: &str) -> ImageData {
    let eroded = morphological_op(input, scalars, false);
    morphological_op(&eroded, scalars, true)
}

/// Morphological closing (dilate then erode) on a binary ImageData field.
///
/// Closing fills small dark holes while preserving the overall shape.
/// Uses a cubic structuring element of radius 1.
pub fn morphological_closing(input: &ImageData, scalars: &str) -> ImageData {
    let dilated = morphological_op(input, scalars, true);
    morphological_op(&dilated, scalars, false)
}

fn morphological_op(input: &ImageData, scalars: &str, dilate: bool) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let n: usize = nx * ny * nz;

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
                let mut best: f64 = if dilate { f64::MIN } else { f64::MAX };

                for dk in -1i64..=1 {
                    let kk: usize = (k as i64 + dk).clamp(0, nz as i64 - 1) as usize;
                    for dj in -1i64..=1 {
                        let jj: usize = (j as i64 + dj).clamp(0, ny as i64 - 1) as usize;
                        for di in -1i64..=1 {
                            let ii: usize = (i as i64 + di).clamp(0, nx as i64 - 1) as usize;
                            let v: f64 = values[kk * ny * nx + jj * nx + ii];
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
    let mut new_attrs = DataSetAttributes::new();
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

    fn make_image_with_dot() -> ImageData {
        // 5x5x1 image with a single bright pixel at center
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![0.0f64; 25];
        values[12] = 1.0; // center pixel
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));
        img
    }

    fn make_image_with_hole() -> ImageData {
        // 5x5x1 image all 1s except a single dark pixel at center
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![1.0f64; 25];
        values[12] = 0.0; // center hole
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));
        img
    }

    #[test]
    fn opening_removes_small_dot() {
        let img = make_image_with_dot();
        let result = morphological_opening(&img, "mask");
        let arr = result.point_data().get_array("mask").unwrap();
        let mut buf = [0.0f64];
        // The single bright pixel should be removed by opening (erode kills it, dilate can't restore)
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 0.0, "opening should remove isolated bright pixel");
    }

    #[test]
    fn closing_fills_small_hole() {
        let img = make_image_with_hole();
        let result = morphological_closing(&img, "mask");
        let arr = result.point_data().get_array("mask").unwrap();
        let mut buf = [0.0f64];
        // The single dark pixel should be filled by closing (dilate fills it, erode can't remove)
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 1.0, "closing should fill isolated dark pixel");
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = morphological_opening(&img, "nonexistent");
        assert_eq!(result.dimensions(), [3, 3, 1]);
    }
}
