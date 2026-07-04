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
    let num_components = arr.num_components();
    if num_components == 0 {
        return input.clone();
    }
    let mut buf = vec![0.0f64; num_components];
    let mut values = Vec::with_capacity(n * num_components);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values.extend(buf.iter().map(|&v| if v >= threshold { 0.0 } else { 1.0 }));
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            output,
            values,
            num_components,
        )));
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
    if arr_a.num_tuples() != arr_b.num_tuples() {
        return a.clone();
    }
    let num_components = arr_a.num_components();
    if num_components == 0 || arr_b.num_components() != num_components {
        return a.clone();
    }
    let n = arr_a.num_tuples();
    let mut ba = vec![0.0f64; num_components];
    let mut bb = vec![0.0f64; num_components];
    let mut values = Vec::with_capacity(n * num_components);
    for i in 0..n {
        arr_a.tuple_as_f64(i, &mut ba);
        arr_b.tuple_as_f64(i, &mut bb);
        values.extend(ba.iter().zip(bb.iter()).map(|(&va, &vb)| {
            if op(va >= threshold, vb >= threshold) {
                1.0
            } else {
                0.0
            }
        }));
    }

    let mut img = a.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            output,
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

    #[test]
    fn logical_ops_process_all_components() {
        let mut a = ImageData::with_dimensions(2, 1, 1);
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "m",
                vec![1.0, 0.0, 0.0, 1.0],
                2,
            )));
        let mut b = ImageData::with_dimensions(2, 1, 1);
        b.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "m",
                vec![1.0, 1.0, 0.0, 0.0],
                2,
            )));

        let r = image_and(&a, &b, "m", 0.5, "out");
        let arr = r.point_data().get_array("out").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0f64, 0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [1.0, 0.0]);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [0.0, 0.0]);
    }

    #[test]
    fn not_processes_all_components() {
        let mut a = ImageData::with_dimensions(1, 1, 1);
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "m",
                vec![1.0, 0.0],
                2,
            )));

        let r = image_not(&a, "m", 0.5, "out");
        let arr = r.point_data().get_array("out").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0f64, 0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [0.0, 1.0]);
    }
}
