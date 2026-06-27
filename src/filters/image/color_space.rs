//! Color space conversions for multi-component image data.

use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::{Scalar, ScalarType};

fn array_from_f64_values(
    name: &str,
    values: Vec<f64>,
    num_components: usize,
    scalar_type: ScalarType,
) -> AnyDataArray {
    fn cast_array<T: Scalar>(name: &str, values: Vec<f64>, num_components: usize) -> AnyDataArray
    where
        AnyDataArray: From<DataArray<T>>,
    {
        AnyDataArray::from(DataArray::from_vec(
            name,
            values.into_iter().map(T::from_f64).collect(),
            num_components,
        ))
    }

    match scalar_type {
        ScalarType::F32 => cast_array::<f32>(name, values, num_components),
        ScalarType::F64 => cast_array::<f64>(name, values, num_components),
        ScalarType::I8 => cast_array::<i8>(name, values, num_components),
        ScalarType::I16 => cast_array::<i16>(name, values, num_components),
        ScalarType::I32 => cast_array::<i32>(name, values, num_components),
        ScalarType::I64 => cast_array::<i64>(name, values, num_components),
        ScalarType::U8 => cast_array::<u8>(name, values, num_components),
        ScalarType::U16 => cast_array::<u16>(name, values, num_components),
        ScalarType::U32 => cast_array::<u32>(name, values, num_components),
        ScalarType::U64 => cast_array::<u64>(name, values, num_components),
    }
}

fn vtk_rgb_to_hsv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let one_third = 1.0 / 3.0;
    let one_sixth = 1.0 / 6.0;
    let two_third = 2.0 / 3.0;

    let mut cmax = r;
    let mut cmin = r;
    if g > cmax {
        cmax = g;
    } else if g < cmin {
        cmin = g;
    }
    if b > cmax {
        cmax = b;
    } else if b < cmin {
        cmin = b;
    }

    let v = cmax;
    let s = if v > 0.0 { (cmax - cmin) / cmax } else { 0.0 };
    let h = if s > 0.0 {
        let mut h = if r == cmax {
            one_sixth * (g - b) / (cmax - cmin)
        } else if g == cmax {
            one_third + one_sixth * (b - r) / (cmax - cmin)
        } else {
            two_third + one_sixth * (r - g) / (cmax - cmin)
        };
        if h < 0.0 {
            h += 1.0;
        }
        h
    } else {
        0.0
    };

    (h, s, v)
}

fn vtk_hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let one_third = 1.0 / 3.0;
    let one_sixth = 1.0 / 6.0;
    let two_third = 2.0 / 3.0;
    let five_sixth = 5.0 / 6.0;

    let (mut r, mut g, mut b) = if h > one_sixth && h <= one_third {
        ((one_third - h) / one_sixth, 1.0, 0.0)
    } else if h > one_third && h <= 0.5 {
        (0.0, 1.0, (h - one_third) / one_sixth)
    } else if h > 0.5 && h <= two_third {
        (0.0, (two_third - h) / one_sixth, 1.0)
    } else if h > two_third && h <= five_sixth {
        ((h - two_third) / one_sixth, 0.0, 1.0)
    } else if h > five_sixth && h <= 1.0 {
        (1.0, 0.0, (1.0 - h) / one_sixth)
    } else {
        (1.0, h / one_sixth, 0.0)
    };

    r = s * r + (1.0 - s);
    g = s * g + (1.0 - s);
    b = s * b + (1.0 - s);

    (r * v, g * v, b * v)
}

/// Convert RGB image (3-component array) to grayscale (1-component).
pub fn rgb_to_grayscale(input: &ImageData, array_name: &str) -> ImageData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 3 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 3];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            0.30 * buf[0] + 0.59 * buf[1] + 0.11 * buf[2]
        })
        .collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(array_from_f64_values(
            "Grayscale",
            data,
            1,
            arr.scalar_type(),
        ))
}

/// Convert RGB to HSV using VTK's default maximum value of 255.
pub fn rgb_to_hsv(input: &ImageData, array_name: &str) -> ImageData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) if a.num_components() >= 3 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let nc = arr.num_components();
    let maximum = 255.0;
    let mut buf = vec![0.0f64; nc];
    let mut data = Vec::with_capacity(n * nc);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let r = buf[0] / maximum;
        let g = buf[1] / maximum;
        let b = buf[2] / maximum;
        let (h, s, v) = vtk_rgb_to_hsv(r, g, b);
        data.push((h * maximum).min(maximum));
        data.push((s * maximum).min(maximum));
        data.push((v * maximum).min(maximum));
        data.extend_from_slice(&buf[3..]);
    }
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(array_from_f64_values("HSV", data, nc, arr.scalar_type()))
}

/// Convert HSV back to RGB using VTK's default maximum value of 255.
pub fn hsv_to_rgb(input: &ImageData, array_name: &str) -> ImageData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) if a.num_components() >= 3 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let nc = arr.num_components();
    let maximum = 255.0;
    let mut buf = vec![0.0f64; nc];
    let mut data = Vec::with_capacity(n * nc);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let h = buf[0] / maximum;
        let s = buf[1] / maximum;
        let v = buf[2] / maximum;
        let (r, g, b) = vtk_hsv_to_rgb(h, s, v);
        data.push((r * maximum).min(maximum));
        data.push((g * maximum).min(maximum));
        data.push((b * maximum).min(maximum));
        data.extend_from_slice(&buf[3..]);
    }
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(array_from_f64_values("RGB", data, nc, arr.scalar_type()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rgb_gray() {
        let dims = [4, 4, 1];
        let data: Vec<f64> = (0..16)
            .flat_map(|i| vec![i as f64 * 16.0, i as f64 * 8.0, i as f64 * 4.0])
            .collect();
        let img = ImageData::with_dimensions(dims[0], dims[1], dims[2])
            .with_spacing([1.0, 1.0, 1.0])
            .with_origin([0.0, 0.0, 0.0])
            .with_point_array(AnyDataArray::F64(DataArray::from_vec("RGB", data, 3)));
        let gray = rgb_to_grayscale(&img, "RGB");
        assert!(gray.point_data().get_array("Grayscale").is_some());
        let arr = gray.point_data().get_array("Grayscale").unwrap();
        assert_eq!(arr.num_components(), 1);
    }
    #[test]
    fn test_rgb_hsv_roundtrip() {
        let data: Vec<f64> = vec![
            255.0, 0.0, 0.0, 0.0, 255.0, 0.0, 0.0, 0.0, 255.0, 128.0, 128.0, 128.0,
        ];
        let img = ImageData::with_dimensions(4, 1, 1)
            .with_spacing([1.0, 1.0, 1.0])
            .with_origin([0.0, 0.0, 0.0])
            .with_point_array(AnyDataArray::F64(DataArray::from_vec(
                "RGB",
                data.clone(),
                3,
            )));
        let hsv = rgb_to_hsv(&img, "RGB");
        let back = hsv_to_rgb(&hsv, "HSV");
        let arr = back.point_data().get_array("RGB").unwrap();
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 255.0).abs() < 1.0); // red
    }

    #[test]
    fn test_rgb_hsv_preserves_extra_components() {
        let img = ImageData::with_dimensions(1, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("RGBA", vec![0.0, 255.0, 0.0, 64.0], 4),
        ));
        let hsv = rgb_to_hsv(&img, "RGBA");
        let arr = hsv.point_data().get_array("HSV").unwrap();
        assert_eq!(arr.num_components(), 4);
        let mut buf = [0.0; 4];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 85.0).abs() < 1e-10);
        assert!((buf[1] - 255.0).abs() < 1e-10);
        assert!((buf[2] - 255.0).abs() < 1e-10);
        assert_eq!(buf[3], 64.0);
    }

    #[test]
    fn test_hsv_to_rgb_matches_vtk_edge_hue_behavior() {
        let img = ImageData::with_dimensions(2, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("HSV", vec![255.0, 255.0, 255.0, -1.0, 255.0, 255.0], 3),
        ));
        let rgb = hsv_to_rgb(&img, "HSV");
        let arr = rgb.point_data().get_array("RGB").unwrap();
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [255.0, 0.0, 0.0]);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 255.0).abs() < 1e-10);
        assert!((buf[1] + 6.0).abs() < 1e-10);
        assert_eq!(buf[2], 0.0);
    }

    #[test]
    fn test_color_conversions_preserve_scalar_type() {
        let rgb = ImageData::with_dimensions(1, 1, 1).with_point_array(AnyDataArray::U8(
            DataArray::from_vec("RGB", vec![255u8, 0, 0], 3),
        ));
        let gray = rgb_to_grayscale(&rgb, "RGB");
        assert_eq!(
            gray.point_data()
                .get_array("Grayscale")
                .unwrap()
                .scalar_type(),
            ScalarType::U8
        );
        let hsv = rgb_to_hsv(&rgb, "RGB");
        assert_eq!(
            hsv.point_data().get_array("HSV").unwrap().scalar_type(),
            ScalarType::U8
        );
        let back = hsv_to_rgb(&hsv, "HSV");
        assert_eq!(
            back.point_data().get_array("RGB").unwrap().scalar_type(),
            ScalarType::U8
        );
    }
}
