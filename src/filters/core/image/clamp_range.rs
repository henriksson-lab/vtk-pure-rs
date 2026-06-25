use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::ScalarType;

/// Window/level adjustment for ImageData (common in medical imaging).
///
/// Maps values through VTK's default window/level color path, producing
/// an RGBA unsigned-char image with no lookup table.
pub fn image_window_level(input: &ImageData, scalars: &str, center: f64, width: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let (lower, upper, lower_val, upper_val) =
        window_level_clamps(arr.scalar_type(), center, width);
    let shift = width / 2.0 - center;
    let scale = if width == 0.0 { 0.0 } else { 255.0 / width };

    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut values = Vec::with_capacity(n * 4);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let result_val = if buf[0] <= lower {
            lower_val
        } else if buf[0] >= upper {
            upper_val
        } else {
            ((buf[0] + shift) * scale).clamp(0.0, 255.0) as u8
        };
        values.push(result_val);
        values.push(result_val);
        values.push(result_val);
        values.push(255);
    }

    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::U8(DataArray::from_vec(
            "WindowLevelColors",
            values,
            4,
        )))
}

fn scalar_type_range(scalar_type: ScalarType) -> [f64; 2] {
    match scalar_type {
        ScalarType::F32 => [f32::MIN as f64, f32::MAX as f64],
        ScalarType::F64 => [f64::MIN, f64::MAX],
        ScalarType::I8 => [i8::MIN as f64, i8::MAX as f64],
        ScalarType::I16 => [i16::MIN as f64, i16::MAX as f64],
        ScalarType::I32 => [i32::MIN as f64, i32::MAX as f64],
        ScalarType::I64 => [i64::MIN as f64, i64::MAX as f64],
        ScalarType::U8 => [u8::MIN as f64, u8::MAX as f64],
        ScalarType::U16 => [u16::MIN as f64, u16::MAX as f64],
        ScalarType::U32 => [u32::MIN as f64, u32::MAX as f64],
        ScalarType::U64 => [u64::MIN as f64, u64::MAX as f64],
    }
}

fn clamp_u8(value: f64) -> u8 {
    if value > 255.0 {
        255
    } else if value < 0.0 {
        0
    } else {
        value as u8
    }
}

fn window_level_clamps(scalar_type: ScalarType, center: f64, width: f64) -> (f64, f64, u8, u8) {
    let f_lower = center - width.abs() / 2.0;
    let f_upper = f_lower + width.abs();
    let range = scalar_type_range(scalar_type);

    let (lower, adjusted_lower) = if f_lower <= range[1] {
        if f_lower >= range[0] {
            (f_lower, f_lower)
        } else {
            (range[0], range[0])
        }
    } else {
        (range[1], range[1])
    };

    let (upper, adjusted_upper) = if f_upper >= range[0] {
        if f_upper <= range[1] {
            (f_upper, f_upper)
        } else {
            (range[1], range[1])
        }
    } else {
        (range[0], range[0])
    };

    let (f_lower_val, f_upper_val) = if width > 0.0 {
        (
            255.0 * (adjusted_lower - f_lower) / width,
            255.0 * (adjusted_upper - f_lower) / width,
        )
    } else if width < 0.0 {
        (
            255.0 + 255.0 * (adjusted_lower - f_lower) / width,
            255.0 + 255.0 * (adjusted_upper - f_lower) / width,
        )
    } else {
        (0.0, 255.0)
    };

    (lower, upper, clamp_u8(f_lower_val), clamp_u8(f_upper_val))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_level() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 25.0, 50.0, 75.0, 100.0],
                1,
            )));

        let result = image_window_level(&img, "v", 50.0, 50.0);
        let arr = result.point_data().get_array("WindowLevelColors").unwrap();
        assert_eq!(arr.num_components(), 4);
        let mut buf = [0.0f64; 4];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [0.0, 0.0, 0.0, 255.0]); // below window
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf, [127.0, 127.0, 127.0, 255.0]); // center
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf, [255.0, 255.0, 255.0, 255.0]); // above window
    }

    #[test]
    fn narrow_window() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![49.0, 50.0, 51.0],
                1,
            )));

        let result = image_window_level(&img, "v", 50.0, 2.0);
        let arr = result.point_data().get_array("WindowLevelColors").unwrap();
        let mut buf = [0.0f64; 4];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 127.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 255.0);
    }

    #[test]
    fn negative_window_inverts() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![49.0, 50.0, 51.0],
                1,
            )));

        let result = image_window_level(&img, "v", 50.0, -2.0);
        let arr = result.point_data().get_array("WindowLevelColors").unwrap();
        let mut buf = [0.0f64; 4];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 255.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 127.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let result = image_window_level(&img, "nope", 0.0, 1.0);
        assert_eq!(result.dimensions(), [3, 1, 1]);
    }

    #[test]
    fn clamps_to_scalar_type_range() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::U8(DataArray::from_vec(
                "v",
                vec![0u8, 127, 255],
                1,
            )));

        let result = image_window_level(&img, "v", 300.0, 200.0);
        let arr = result.point_data().get_array("WindowLevelColors").unwrap();
        let mut buf = [0.0f64; 4];
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 70.0);
    }
}
