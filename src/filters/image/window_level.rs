//! Window/level contrast adjustment for scalar ImageData (common in medical imaging).

use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::ScalarType;

/// Apply window/level contrast to a scalar array.
///
/// Values are mapped with VTK's no-lookup-table window/level formula
/// to unsigned char values in [0, 255].
pub fn window_level(image: &ImageData, array_name: &str, center: f64, width: f64) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];
    if arr.num_tuples() < n {
        return image.clone();
    }
    let (lower, upper, lower_val, upper_val) =
        window_level_clamps(arr.scalar_type(), center, width);
    let shift = width / 2.0 - center;
    let scale = if width == 0.0 { 0.0 } else { 255.0 / width };
    let mut buf = [0.0f64];
    let data: Vec<u8> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let value = buf[0];
            if value <= lower {
                lower_val
            } else if value >= upper {
                upper_val
            } else {
                ((value + shift) * scale).clamp(0.0, 255.0) as u8
            }
        })
        .collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::U8(DataArray::from_vec(array_name, data, 1)));
    result
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

/// Auto window/level: use percentile-based window.
pub fn auto_window_level(
    image: &ImageData,
    array_name: &str,
    low_pct: f64,
    high_pct: f64,
) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let n = arr.num_tuples();
    if n == 0 {
        return image.clone();
    }
    let mut buf = [0.0f64];
    let mut values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let lo_idx = (low_pct * (n - 1) as f64) as usize;
    let hi_idx = (high_pct * (n - 1) as f64) as usize;
    let lo = values[lo_idx.min(n - 1)];
    let hi = values[hi_idx.min(n - 1)];
    let center = (lo + hi) / 2.0;
    let width = (hi - lo).max(1e-15);
    window_level(image, array_name, center, width)
}

/// Gamma correction on a scalar array.
pub fn gamma_correction(image: &ImageData, array_name: &str, gamma: f64) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];
    if n == 0 || arr.num_tuples() < n {
        return image.clone();
    }
    let mut buf = [0.0f64];
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
    }
    let range = (max_v - min_v).max(1e-15);
    let data: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let t = ((buf[0] - min_v) / range).clamp(0.0, 1.0);
            min_v + t.powf(gamma) * range
        })
        .collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

/// Sigmoid contrast enhancement.
pub fn sigmoid_contrast(image: &ImageData, array_name: &str, alpha: f64, beta: f64) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];
    if arr.num_tuples() < n {
        return image.clone();
    }
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            1.0 / (1.0 + (-alpha * (buf[0] - beta)).exp())
        })
        .collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wl() {
        let img = ImageData::from_function(
            [10, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x * 10.0,
        );
        let result = window_level(&img, "v", 50.0, 40.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] <= 0.01); // below window
    }
    #[test]
    fn auto_wl() {
        let img = ImageData::from_function(
            [100, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = auto_window_level(&img, "v", 0.05, 0.95);
        assert!(result.point_data().get_array("v").is_some());
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

        let result = window_level(&img, "v", 300.0, 200.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 70.0);
    }
    #[test]
    fn gamma() {
        let img = ImageData::from_function(
            [10, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x / 9.0,
        );
        let result = gamma_correction(&img, "v", 2.0);
        assert!(result.point_data().get_array("v").is_some());
    }
    #[test]
    fn sigmoid() {
        let img = ImageData::from_function(
            [10, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = sigmoid_contrast(&img, "v", 1.0, 5.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(5, &mut buf);
        assert!((buf[0] - 0.5).abs() < 0.01); // sigmoid midpoint at beta=5
    }
}
