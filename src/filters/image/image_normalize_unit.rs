//! Normalize scalar-component vectors per point.
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_normalize_unit(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let max_c = arr.num_components();
    let mut buf = vec![0.0f64; max_c];
    let mut data = Vec::with_capacity(n * max_c);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let mut sum = 0.0f32;
        for value in &buf {
            let value = *value as f32;
            sum += value * value;
        }
        if sum > 0.0 {
            sum = 1.0 / sum.sqrt();
        }
        for value in &buf {
            data.push((*value as f32) * sum);
        }
    }
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F32(DataArray::from_vec(scalars, data, max_c)))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_normalize_unit(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn normalizes_each_tuple_vector() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![3.0, 4.0, 0.0, 0.0, 0.0, 5.0],
                3,
            )));

        let r = image_normalize_unit(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 3);
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.6).abs() < 1e-6);
        assert!((buf[1] - 0.8).abs() < 1e-6);
        assert!((buf[2]).abs() < 1e-6);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0]).abs() < 1e-6);
        assert!((buf[1]).abs() < 1e-6);
        assert!((buf[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn leaves_zero_magnitude_tuple_zero() {
        let mut img = ImageData::with_dimensions(1, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.0, 0.0],
                3,
            )));

        let r = image_normalize_unit(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [1.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [0.0, 0.0, 0.0]);
    }
}
