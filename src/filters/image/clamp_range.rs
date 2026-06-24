use crate::data::{AnyDataArray, DataArray, ImageData};

/// Window/level adjustment for ImageData (common in medical imaging).
///
/// Maps values in [center - width/2, center + width/2] to [0, 1].
/// Values below the window are 0, above are 1.
pub fn image_window_level(input: &ImageData, scalars: &str, center: f64, width: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let half = width * 0.5;
    let lo = center - half;
    let hi = center + half;
    let range = (hi - lo).max(1e-15);

    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            ((buf[0] - lo) / range).clamp(0.0, 1.0)
        })
        .collect();

    let mut img = input.clone();
    let mut new_attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                values.clone(),
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
    fn window_level() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 25.0, 50.0, 75.0, 100.0],
                1,
            )));

        let result = image_window_level(&img, "v", 50.0, 50.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // below window
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-10); // center
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 1.0); // above window
    }

    #[test]
    fn narrow_window() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![49.0, 50.0, 51.0],
                1,
            )));

        let result = image_window_level(&img, "v", 50.0, 2.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-10);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let result = image_window_level(&img, "nope", 0.0, 1.0);
        assert_eq!(result.dimensions(), [3, 1, 1]);
    }
}
