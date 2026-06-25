use crate::data::{AnyDataArray, DataArray, ImageData};

/// Add two ImageData scalar fields element-wise.
pub fn image_add(a: &ImageData, b: &ImageData, scalars: &str, output_name: &str) -> ImageData {
    image_binary_op(a, b, scalars, output_name, |x, y| x + y)
}

/// Subtract ImageData scalar fields element-wise (a - b).
pub fn image_subtract(a: &ImageData, b: &ImageData, scalars: &str, output_name: &str) -> ImageData {
    image_binary_op(a, b, scalars, output_name, |x, y| x - y)
}

/// Multiply two ImageData scalar fields element-wise.
pub fn image_multiply(a: &ImageData, b: &ImageData, scalars: &str, output_name: &str) -> ImageData {
    image_binary_op(a, b, scalars, output_name, |x, y| x * y)
}

/// Scale an ImageData scalar field by a constant.
pub fn image_scale(input: &ImageData, scalars: &str, factor: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let num_components = arr.num_components();
    let n = arr.num_tuples();
    let mut values = Vec::with_capacity(n * num_components);
    let mut buf = vec![0.0f64; num_components];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values.extend(buf.iter().map(|&x| x * factor));
    }

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

fn image_binary_op<F>(
    a: &ImageData,
    b: &ImageData,
    scalars: &str,
    output_name: &str,
    op: F,
) -> ImageData
where
    F: Fn(f64, f64) -> f64,
{
    let arr_a = match a.point_data().get_array(scalars) {
        Some(x) => x,
        None => return a.clone(),
    };
    let arr_b = match b.point_data().get_array(scalars) {
        Some(x) if x.num_components() == arr_a.num_components() => x,
        None => return a.clone(),
        _ => return a.clone(),
    };

    let num_components = arr_a.num_components();
    let n = arr_a.num_tuples().min(arr_b.num_tuples());
    let mut values = Vec::with_capacity(n * num_components);
    let mut ba = vec![0.0f64; num_components];
    let mut bb = vec![0.0f64; num_components];
    for i in 0..n {
        arr_a.tuple_as_f64(i, &mut ba);
        arr_b.tuple_as_f64(i, &mut bb);
        values.extend(ba.iter().zip(&bb).map(|(&x, &y)| op(x, y)));
    }

    let mut img = a.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            output_name,
            values,
            num_components,
        )));
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
    fn add_images() {
        let a = make_img(vec![1.0, 2.0, 3.0]);
        let b = make_img(vec![10.0, 20.0, 30.0]);
        let result = image_add(&a, &b, "v", "sum");
        let arr = result.point_data().get_array("sum").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 11.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 33.0);
    }

    #[test]
    fn subtract_images() {
        let a = make_img(vec![10.0, 20.0]);
        let b = make_img(vec![3.0, 5.0]);
        let result = image_subtract(&a, &b, "v", "diff");
        let arr = result.point_data().get_array("diff").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 7.0);
    }

    #[test]
    fn scale_image() {
        let img = make_img(vec![2.0, 4.0, 6.0]);
        let result = image_scale(&img, "v", 0.5);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 2.0);
    }

    #[test]
    fn multiply_images() {
        let a = make_img(vec![2.0, 3.0]);
        let b = make_img(vec![5.0, 7.0]);
        let result = image_multiply(&a, &b, "v", "prod");
        let arr = result.point_data().get_array("prod").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 10.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 21.0);
    }
}
