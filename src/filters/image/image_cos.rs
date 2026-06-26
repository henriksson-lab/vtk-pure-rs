//! Cosine of pixel values

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Cosine of pixel values
pub fn image_cos(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let num_components = arr.num_components();
    let mut buf = vec![0.0f64; num_components];
    let mut data = Vec::with_capacity(n * num_components);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        data.extend(buf.iter().map(|value| value.cos()));
    }
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            scalars,
            data,
            num_components,
        )))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_image_cos() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_cos(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn test_image_cos_multi_component() {
        let img = ImageData::with_dimensions(2, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec(
                "v",
                vec![0.0, std::f64::consts::FRAC_PI_2, std::f64::consts::PI, 2.0],
                2,
            ),
        ));

        let r = image_cos(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut tuple = [0.0; 2];
        arr.tuple_as_f64(0, &mut tuple);
        assert_eq!(arr.num_components(), 2);
        assert!((tuple[0] - 1.0).abs() < 1e-12);
        assert!(tuple[1].abs() < 1e-12);
        arr.tuple_as_f64(1, &mut tuple);
        assert!((tuple[0] + 1.0).abs() < 1e-12);
        assert!((tuple[1] - 2.0f64.cos()).abs() < 1e-12);
    }
}
