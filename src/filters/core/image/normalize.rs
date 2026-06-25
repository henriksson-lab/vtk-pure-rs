use crate::data::{AnyDataArray, DataArray, ImageData};

/// Normalize the scalar components at each image point to unit length.
pub fn image_normalize(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    let nc = arr.num_components();
    let mut buf = vec![0.0f64; nc];
    let mut values = Vec::with_capacity(n * nc);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let sum: f64 = buf.iter().map(|v| v * v).sum();
        let scale = if sum > 0.0 { 1.0 / sum.sqrt() } else { 0.0 };
        values.extend(buf.iter().map(|v| v * scale));
    }

    let mut img = input.clone();
    let mut new_attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                values.clone(),
                nc,
            )));
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = new_attrs;
    img
}

/// Invert an ImageData scalar field: out = max - value.
pub fn image_invert(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut max_v = f64::NEG_INFINITY;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        max_v = max_v.max(buf[0]);
    }

    let mut values = Vec::with_capacity(n);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values.push(max_v - buf[0]);
    }

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
    fn normalize_vectors_per_point() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![3.0, 4.0, 0.0, 0.0, 0.0, 0.0, -6.0, 8.0, 0.0],
                3,
            )));

        let result = image_normalize(&img, "v");
        let arr = result.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 3);
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.6).abs() < 1e-10);
        assert!((buf[1] - 0.8).abs() < 1e-10);
        assert!((buf[2] - 0.0).abs() < 1e-10);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [0.0, 0.0, 0.0]);
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] + 0.6).abs() < 1e-10);
        assert!((buf[1] - 0.8).abs() < 1e-10);
        assert!((buf[2] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn invert_values() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 5.0, 10.0],
                1,
            )));

        let result = image_invert(&img, "v");
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 10.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let result = image_normalize(&img, "nope");
        assert_eq!(result.dimensions(), [3, 1, 1]);
    }
}
