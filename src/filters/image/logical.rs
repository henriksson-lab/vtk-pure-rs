use crate::data::{AnyDataArray, DataArray, ImageData};

/// Logical AND of two binary ImageData fields.
/// Output is 1.0 where both inputs are >= threshold, 0.0 otherwise.
pub fn image_and(
    a: &ImageData,
    b: &ImageData,
    scalars: &str,
    threshold: f64,
    output: &str,
) -> ImageData {
    image_logical_op(a, b, scalars, threshold, output, |va, vb| va && vb)
}

/// Logical OR of two binary ImageData fields.
pub fn image_or(
    a: &ImageData,
    b: &ImageData,
    scalars: &str,
    threshold: f64,
    output: &str,
) -> ImageData {
    image_logical_op(a, b, scalars, threshold, output, |va, vb| va || vb)
}

/// Logical XOR of two binary ImageData fields.
pub fn image_xor(
    a: &ImageData,
    b: &ImageData,
    scalars: &str,
    threshold: f64,
    output: &str,
) -> ImageData {
    image_logical_op(a, b, scalars, threshold, output, |va, vb| va ^ vb)
}

/// Logical NOT of a binary ImageData field.
pub fn image_not(input: &ImageData, scalars: &str, threshold: f64, output: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] >= threshold {
                0.0
            } else {
                1.0
            }
        })
        .collect();

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(output, values, 1)));
    img
}

fn image_logical_op<F>(
    a: &ImageData,
    b: &ImageData,
    scalars: &str,
    threshold: f64,
    output: &str,
    op: F,
) -> ImageData
where
    F: Fn(bool, bool) -> bool,
{
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
            if op(ba[0] >= threshold, bb[0] >= threshold) {
                1.0
            } else {
                0.0
            }
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
            .add_array(AnyDataArray::F64(DataArray::from_vec("m", vals, 1)));
        img
    }

    #[test]
    fn and_op() {
        let a = make_img(vec![1.0, 1.0, 0.0, 0.0]);
        let b = make_img(vec![1.0, 0.0, 1.0, 0.0]);
        let r = image_and(&a, &b, "m", 0.5, "out");
        let arr = r.point_data().get_array("out").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn or_op() {
        let a = make_img(vec![1.0, 0.0, 0.0]);
        let b = make_img(vec![0.0, 1.0, 0.0]);
        let r = image_or(&a, &b, "m", 0.5, "out");
        let arr = r.point_data().get_array("out").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn not_op() {
        let a = make_img(vec![1.0, 0.0, 1.0]);
        let r = image_not(&a, "m", 0.5, "out");
        let arr = r.point_data().get_array("out").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn xor_op() {
        let a = make_img(vec![1.0, 1.0, 0.0, 0.0]);
        let b = make_img(vec![1.0, 0.0, 1.0, 0.0]);
        let r = image_xor(&a, &b, "m", 0.5, "out");
        let arr = r.point_data().get_array("out").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // both true
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
