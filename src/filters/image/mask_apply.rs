use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply a mask to an ImageData: zero out voxels where mask < threshold.
pub fn image_apply_mask(
    input: &ImageData,
    scalars: &str,
    mask_name: &str,
    threshold: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let mask = match input.point_data().get_array(mask_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let n = arr.num_tuples().min(mask.num_tuples());
    let mut ba = [0.0f64];
    let mut bm = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut ba);
            mask.tuple_as_f64(i, &mut bm);
            if bm[0] >= threshold {
                ba[0]
            } else {
                0.0
            }
        })
        .collect();

    let mut img = input.clone();
    let mut attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                values.clone(),
                1,
            )));
        } else {
            attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = attrs;
    img
}

/// Create a mask from a scalar condition: 1 where condition is true, 0 otherwise.
pub fn image_create_mask(
    input: &ImageData,
    scalars: &str,
    min_val: f64,
    max_val: f64,
    output: &str,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mask: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] >= min_val && buf[0] <= max_val {
                1.0
            } else {
                0.0
            }
        })
        .collect();

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(output, mask, 1)));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_mask() {
        let mut img = ImageData::with_dimensions(4, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0, 20.0, 30.0, 40.0],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "m",
                vec![1.0, 0.0, 1.0, 0.0],
                1,
            )));

        let result = image_apply_mask(&img, "v", "m", 0.5);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 10.0); // mask=1
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0); // mask=0
    }

    #[test]
    fn create_mask() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 3.0, 5.0, 7.0, 9.0],
                1,
            )));

        let result = image_create_mask(&img, "v", 3.0, 7.0, "m");
        let arr = result.point_data().get_array("m").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // 1 < 3
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0); // 3 in [3,7]
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 0.0); // 9 > 7
    }

    #[test]
    fn missing_arrays() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = image_apply_mask(&img, "v", "m", 0.5);
        assert_eq!(r.dimensions(), [3, 1, 1]);
    }
}
