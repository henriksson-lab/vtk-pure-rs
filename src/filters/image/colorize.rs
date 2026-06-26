use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply a colormap to a scalar ImageData, producing RGB output.
///
/// Maps scalar values through a simple blue-to-red colormap and
/// stores the result as a 3-component "RGB" array.
pub fn image_colorize(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
    }
    let range = (max_v - min_v).max(1e-15);

    let mut rgb = Vec::with_capacity(n * 3);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let t = (buf[0] - min_v) / range;
        // Blue -> White -> Red
        if t < 0.5 {
            let s = t * 2.0;
            rgb.push(s);
            rgb.push(s);
            rgb.push(1.0);
        } else {
            let s = (t - 0.5) * 2.0;
            rgb.push(1.0);
            rgb.push(1.0 - s);
            rgb.push(1.0 - s);
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("RGB", rgb, 3)));
    img
}

/// Apply a custom lookup table to a scalar ImageData.
///
/// `lut` maps scalar values to RGB colors. Fractional scalar values are
/// linearly interpolated between adjacent lookup-table entries.
pub fn image_apply_lut(input: &ImageData, scalars: &str, lut: &[[f64; 3]]) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    let lut_len = lut.len();
    if lut_len == 0 {
        return input.clone();
    }

    let mut buf = [0.0f64];
    let mut rgb = Vec::with_capacity(n * 3);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let x = buf[0].clamp(0.0, (lut_len - 1) as f64);
        let idx = x.floor() as usize;
        let next_idx = (idx + 1).min(lut_len - 1);
        let t = x - idx as f64;
        let c0 = lut[idx];
        let c1 = lut[next_idx];
        rgb.push(c0[0] + t * (c1[0] - c0[0]));
        rgb.push(c0[1] + t * (c1[1] - c0[1]));
        rgb.push(c0[2] + t * (c1[2] - c0[2]));
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("RGB", rgb, 3)));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colorize_gradient() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.25, 0.5, 0.75, 1.0],
                1,
            )));
        let result = image_colorize(&img, "v");
        let arr = result.point_data().get_array("RGB").unwrap();
        assert_eq!(arr.num_components(), 3);
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[2] > 0.9); // blue at low
        arr.tuple_as_f64(4, &mut buf);
        assert!(buf[0] > 0.9); // red at high
    }

    #[test]
    fn lut_basic() {
        let lut = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 1.0, 2.0],
                1,
            )));
        let result = image_apply_lut(&img, "v", &lut);
        let arr = result.point_data().get_array("RGB").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [1.0, 0.0, 0.0]);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn lut_interpolates_fractional_indices() {
        let lut = [[0.0, 0.0, 0.0], [1.0, 0.5, 0.0]];
        let mut img = ImageData::with_dimensions(1, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![0.5], 1)));
        let result = image_apply_lut(&img, "v", &lut);
        let arr = result.point_data().get_array("RGB").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [0.5, 0.25, 0.0]);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = image_colorize(&img, "nope");
        assert!(r.point_data().get_array("RGB").is_none());
    }
}
