use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::ScalarType;

/// Alpha-blend two ImageData fields.
///
/// This follows vtkImageBlend normal mode for two inputs: the first image is
/// copied to the output, then the second image is blended over it with
/// `opacity`.
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
    let out_components = arr_a.num_components();
    let in_components = arr_b.num_components();
    if out_components == 0 || in_components == 0 {
        return a.clone();
    }
    let n = arr_a.num_tuples().min(arr_b.num_tuples());
    let opacity = alpha.clamp(0.0, 1.0);
    let (min_alpha, max_alpha) = scalar_alpha_range(arr_b.scalar_type());
    let alpha_scale = opacity / (max_alpha - min_alpha);
    let mut out_tuple = vec![0.0f64; out_components];
    let mut in_tuple = vec![0.0f64; in_components];
    let mut values = Vec::with_capacity(n * out_components);

    for i in 0..n {
        arr_a.tuple_as_f64(i, &mut out_tuple);
        arr_b.tuple_as_f64(i, &mut in_tuple);

        if out_components == 4 && in_components == 4 {
            let r = alpha_scale * (in_tuple[3] - min_alpha);
            let f = 1.0 - r;
            out_tuple[0] = out_tuple[0] * f + in_tuple[0] * r;
            out_tuple[1] = out_tuple[1] * f + in_tuple[1] * r;
            out_tuple[2] = out_tuple[2] * f + in_tuple[2] * r;
        } else if out_components >= 3 && in_components >= 4 {
            let r = alpha_scale * (in_tuple[3] - min_alpha);
            let f = 1.0 - r;
            out_tuple[0] = out_tuple[0] * f + in_tuple[0] * r;
            out_tuple[1] = out_tuple[1] * f + in_tuple[1] * r;
            out_tuple[2] = out_tuple[2] * f + in_tuple[2] * r;
        } else if out_components >= 3 && in_components == 3 {
            let f = 1.0 - opacity;
            out_tuple[0] = out_tuple[0] * f + in_tuple[0] * opacity;
            out_tuple[1] = out_tuple[1] * f + in_tuple[1] * opacity;
            out_tuple[2] = out_tuple[2] * f + in_tuple[2] * opacity;
        } else if out_components >= 3 && in_components == 2 {
            let r = alpha_scale * (in_tuple[1] - min_alpha);
            let f = 1.0 - r;
            out_tuple[0] = out_tuple[0] * f + in_tuple[0] * r;
            out_tuple[1] = out_tuple[1] * f + in_tuple[0] * r;
            out_tuple[2] = out_tuple[2] * f + in_tuple[0] * r;
        } else if out_components >= 3 && in_components == 1 {
            let f = 1.0 - opacity;
            out_tuple[0] = out_tuple[0] * f + in_tuple[0] * opacity;
            out_tuple[1] = out_tuple[1] * f + in_tuple[0] * opacity;
            out_tuple[2] = out_tuple[2] * f + in_tuple[0] * opacity;
        } else if in_components == 2 {
            let r = alpha_scale * (in_tuple[1] - min_alpha);
            let f = 1.0 - r;
            out_tuple[0] = out_tuple[0] * f + in_tuple[0] * r;
        } else {
            let f = 1.0 - opacity;
            out_tuple[0] = out_tuple[0] * f + in_tuple[0] * opacity;
        }

        values.extend_from_slice(&out_tuple);
    }

    let mut img = a.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            output,
            values,
            out_components,
        )));
    img
}

fn scalar_alpha_range(scalar_type: ScalarType) -> (f64, f64) {
    match scalar_type {
        ScalarType::F32 | ScalarType::F64 => (0.0, 1.0),
        ScalarType::I8 => (i8::MIN as f64, i8::MAX as f64),
        ScalarType::I16 => (i16::MIN as f64, i16::MAX as f64),
        ScalarType::I32 => (i32::MIN as f64, i32::MAX as f64),
        ScalarType::I64 => (i64::MIN as f64, i64::MAX as f64),
        ScalarType::U8 => (u8::MIN as f64, u8::MAX as f64),
        ScalarType::U16 => (u16::MIN as f64, u16::MAX as f64),
        ScalarType::U32 => (u32::MIN as f64, u32::MAX as f64),
        ScalarType::U64 => (u64::MIN as f64, u64::MAX as f64),
    }
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
    fn blend_all_b() {
        let a = make_img(vec![100.0]);
        let b = make_img(vec![0.0]);
        let r = image_blend(&a, &b, "v", 1.0, "out");
        let arr = r.point_data().get_array("out").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn rgba_uses_foreground_alpha_and_preserves_background_alpha() {
        let mut a = ImageData::with_dimensions(1, 1, 1);
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "rgba",
                vec![0.0, 0.0, 0.0, 0.25],
                4,
            )));
        let mut b = ImageData::with_dimensions(1, 1, 1);
        b.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "rgba",
                vec![1.0, 0.0, 0.0, 0.5],
                4,
            )));

        let r = image_blend(&a, &b, "rgba", 1.0, "out");
        let arr = r.point_data().get_array("out").unwrap();
        let mut buf = [0.0f64; 4];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [0.5, 0.0, 0.0, 0.25]);
    }

    #[test]
    fn unsigned_char_alpha_is_scaled_like_vtk() {
        let mut a = ImageData::with_dimensions(1, 1, 1);
        a.point_data_mut()
            .add_array(AnyDataArray::U8(DataArray::from_vec(
                "rgba",
                vec![0, 0, 0, 64],
                4,
            )));
        let mut b = ImageData::with_dimensions(1, 1, 1);
        b.point_data_mut()
            .add_array(AnyDataArray::U8(DataArray::from_vec(
                "rgba",
                vec![255, 0, 0, 128],
                4,
            )));

        let r = image_blend(&a, &b, "rgba", 1.0, "out");
        let arr = r.point_data().get_array("out").unwrap();
        let mut buf = [0.0f64; 4];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 128.0).abs() < 1.0);
        assert_eq!(buf[3], 64.0);
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
