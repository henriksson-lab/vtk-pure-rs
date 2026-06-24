use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute element-wise minimum of two ImageData scalar fields.
pub fn image_min(a: &ImageData, b: &ImageData, scalars: &str, output: &str) -> ImageData {
    image_binary(a, b, scalars, output, f64::min)
}

/// Compute element-wise maximum of two ImageData scalar fields.
pub fn image_max(a: &ImageData, b: &ImageData, scalars: &str, output: &str) -> ImageData {
    image_binary(a, b, scalars, output, f64::max)
}

/// Compute element-wise difference magnitude |a - b|.
pub fn image_diff(a: &ImageData, b: &ImageData, scalars: &str, output: &str) -> ImageData {
    image_binary(a, b, scalars, output, |x, y| (x - y).abs())
}

fn image_binary<F: Fn(f64, f64) -> f64>(
    a: &ImageData,
    b: &ImageData,
    scalars: &str,
    output: &str,
    op: F,
) -> ImageData {
    let arr_a = match a.point_data().get_array(scalars) {
        Some(x) => x,
        None => return a.clone(),
    };
    let arr_b = match b.point_data().get_array(scalars) {
        Some(x) => x,
        None => return a.clone(),
    };
    let n = arr_a.num_tuples().min(arr_b.num_tuples());
    let mut ba = [0.0f64];
    let mut bb = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr_a.tuple_as_f64(i, &mut ba);
            arr_b.tuple_as_f64(i, &mut bb);
            op(ba[0], bb[0])
        })
        .collect();

    let mut img = a.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(output, values, 1)));
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
    fn min_op() {
        let a = make_img(vec![1.0, 5.0, 3.0]);
        let b = make_img(vec![2.0, 4.0, 6.0]);
        let r = image_min(&a, &b, "v", "out");
        let arr = r.point_data().get_array("out").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 4.0);
    }

    #[test]
    fn max_op() {
        let a = make_img(vec![1.0, 5.0]);
        let b = make_img(vec![2.0, 4.0]);
        let r = image_max(&a, &b, "v", "out");
        let arr = r.point_data().get_array("out").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 2.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 5.0);
    }

    #[test]
    fn diff_op() {
        let a = make_img(vec![10.0, 3.0]);
        let b = make_img(vec![7.0, 8.0]);
        let r = image_diff(&a, &b, "v", "out");
        let arr = r.point_data().get_array("out").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 3.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 5.0);
    }
}
