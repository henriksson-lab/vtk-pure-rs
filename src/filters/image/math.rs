use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::filters::image::image_math_ops;

/// Add two ImageData scalar fields element-wise, storing the result in `output_name`.
///
/// Thin wrapper over [`image_math_ops::image_add`] (VTK_ADD of `vtkImageMathematics`)
/// that writes the result to a separate array instead of replacing `scalars`.
pub fn image_add(a: &ImageData, b: &ImageData, scalars: &str, output_name: &str) -> ImageData {
    named_binary(a, b, scalars, output_name, image_math_ops::image_add)
}

/// Subtract ImageData scalar fields element-wise (a - b), storing the result in `output_name`.
pub fn image_subtract(a: &ImageData, b: &ImageData, scalars: &str, output_name: &str) -> ImageData {
    named_binary(a, b, scalars, output_name, image_math_ops::image_subtract)
}

/// Multiply two ImageData scalar fields element-wise, storing the result in `output_name`.
pub fn image_multiply(a: &ImageData, b: &ImageData, scalars: &str, output_name: &str) -> ImageData {
    named_binary(a, b, scalars, output_name, image_math_ops::image_multiply)
}

/// Scale an ImageData scalar field by a constant.
///
/// This is VTK_MULTIPLYBYK of `vtkImageMathematics`
/// (VTK/Imaging/Math/vtkImageMathematics.cxx:242), implemented once in
/// [`image_math_ops::image_multiply_by_k`].
pub fn image_scale(input: &ImageData, scalars: &str, factor: f64) -> ImageData {
    image_math_ops::image_multiply_by_k(input, scalars, factor)
}

/// Run a two-input `image_math_ops` operation and store its result under `output_name`
/// instead of overwriting the `scalars` array.
fn named_binary<F>(
    a: &ImageData,
    b: &ImageData,
    scalars: &str,
    output_name: &str,
    op: F,
) -> ImageData
where
    F: Fn(&ImageData, &ImageData, &str) -> ImageData,
{
    let (n, nc) = match (
        a.point_data().get_array(scalars),
        b.point_data().get_array(scalars),
    ) {
        (Some(x), Some(y))
            if x.num_tuples() == y.num_tuples() && x.num_components() == y.num_components() =>
        {
            (x.num_tuples(), x.num_components())
        }
        _ => return a.clone(),
    };

    let computed = op(a, b, scalars);
    let arr = match computed.point_data().get_array(scalars) {
        Some(x) => x,
        None => return a.clone(),
    };
    let mut values = Vec::with_capacity(n * nc);
    let mut buf = vec![0.0f64; nc];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values.extend(buf.iter().copied());
    }

    let mut img = a.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            output_name,
            values,
            nc,
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

    #[test]
    fn binary_ops_preserve_components() {
        let mut a = ImageData::with_dimensions(2, 1, 1);
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0],
                2,
            )));
        let mut b = ImageData::with_dimensions(2, 1, 1);
        b.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0, 20.0, 30.0, 40.0],
                2,
            )));

        let result = image_add(&a, &b, "v", "sum");
        let arr = result.point_data().get_array("sum").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0f64, 0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [11.0, 22.0]);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [33.0, 44.0]);
    }

    #[test]
    fn scale_preserves_components() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![2.0, 4.0, 6.0, 8.0],
                2,
            )));

        let result = image_scale(&img, "v", 0.5);
        let arr = result.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0f64, 0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [1.0, 2.0]);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [3.0, 4.0]);
    }
}
