use crate::data::{AnyDataArray, DataArray, ImageData};

/// Alpha-blend two ImageData fields.
///
/// result = alpha * a + (1 - alpha) * b.
pub fn image_blend(
    a: &ImageData,
    b: &ImageData,
    scalars: &str,
    alpha: f64,
    output: &str,
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
    let al = alpha.clamp(0.0, 1.0);
    let mut ba = [0.0f64];
    let mut bb = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr_a.tuple_as_f64(i, &mut ba);
            arr_b.tuple_as_f64(i, &mut bb);
            al * ba[0] + (1.0 - al) * bb[0]
        })
        .collect();
    let mut img = a.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(output, values, 1)));
    img
}

/// Weighted blend of multiple ImageData fields.
///
/// result = Σ(weight_i * field_i) / Σ(weight_i).
pub fn image_weighted_blend(
    inputs: &[(&ImageData, f64)],
    scalars: &str,
    output: &str,
) -> Option<ImageData> {
    if inputs.is_empty() {
        return None;
    }
    let first = inputs[0].0;
    let n = match first.point_data().get_array(scalars) {
        Some(a) => a.num_tuples(),
        None => return None,
    };

    let mut result = vec![0.0f64; n];
    let mut total_w = 0.0;
    let mut buf = [0.0f64];

    for &(img, w) in inputs {
        if let Some(arr) = img.point_data().get_array(scalars) {
            let m = arr.num_tuples().min(n);
            for i in 0..m {
                arr.tuple_as_f64(i, &mut buf);
                result[i] += buf[0] * w;
            }
            total_w += w;
        }
    }
    if total_w > 1e-15 {
        for v in &mut result {
            *v /= total_w;
        }
    }

    let mut img = first.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(output, result, 1)));
    Some(img)
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
    fn blend_50_50() {
        let a = make_img(vec![0.0, 10.0]);
        let b = make_img(vec![10.0, 0.0]);
        let r = image_blend(&a, &b, "v", 0.5, "out");
        let arr = r.point_data().get_array("out").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn blend_all_a() {
        let a = make_img(vec![100.0]);
        let b = make_img(vec![0.0]);
        let r = image_blend(&a, &b, "v", 1.0, "out");
        let arr = r.point_data().get_array("out").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 100.0);
    }

    #[test]
    fn weighted_blend_test() {
        let a = make_img(vec![10.0]);
        let b = make_img(vec![30.0]);
        let r = image_weighted_blend(&[(&a, 1.0), (&b, 3.0)], "v", "out").unwrap();
        let arr = r.point_data().get_array("out").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 25.0).abs() < 1e-10); // (10+90)/4
    }

    #[test]
    fn missing_array() {
        let a = make_img(vec![1.0]);
        let b = make_img(vec![2.0]);
        let r = image_blend(&a, &b, "nope", 0.5, "out");
        assert!(r.point_data().get_array("out").is_none());
    }
}
