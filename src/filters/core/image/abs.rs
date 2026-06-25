use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute absolute value of an ImageData scalar field.
pub fn image_abs(input: &ImageData, scalars: &str) -> ImageData {
    image_unary_op(input, scalars, |v| v.abs())
}

/// Compute square root of an ImageData scalar field.
pub fn image_sqrt(input: &ImageData, scalars: &str) -> ImageData {
    image_unary_op(input, scalars, |v| v.sqrt())
}

/// Compute log (natural) of an ImageData scalar field.
pub fn image_log(input: &ImageData, scalars: &str) -> ImageData {
    image_unary_op(input, scalars, |v| v.ln())
}

/// Compute exponential of an ImageData scalar field.
pub fn image_exp(input: &ImageData, scalars: &str) -> ImageData {
    image_unary_op(input, scalars, |v| v.exp())
}

/// Clamp ImageData values to [min, max].
pub fn image_clamp(input: &ImageData, scalars: &str, min_val: f64, max_val: f64) -> ImageData {
    image_unary_op(input, scalars, |v| v.clamp(min_val, max_val))
}

fn image_unary_op<F: Fn(f64) -> f64>(input: &ImageData, scalars: &str, op: F) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    let num_components = arr.num_components();
    let mut buf = vec![0.0f64; num_components];
    let values: Vec<f64> = (0..n)
        .flat_map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf.iter().map(|&v| op(v)).collect::<Vec<_>>()
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

    fn make_img(vals: Vec<f64>) -> ImageData {
        let n = vals.len();
        let mut img = ImageData::with_dimensions(n, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vals, 1)));
        img
    }

    #[test]
    fn abs_values() {
        let img = make_img(vec![-3.0, 0.0, 5.0]);
        let r = image_abs(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 3.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 5.0);
    }

    #[test]
    fn sqrt_values() {
        let img = make_img(vec![4.0, 9.0, 16.0]);
        let r = image_sqrt(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn clamp_values() {
        let img = make_img(vec![-10.0, 0.5, 10.0]);
        let r = image_clamp(&img, "v", 0.0, 1.0);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.5);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn missing_array() {
        let img = make_img(vec![1.0]);
        let r = image_abs(&img, "nope");
        assert!(r.point_data().get_array("v").is_some()); // unchanged
    }
}
