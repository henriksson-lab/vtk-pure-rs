//! Copy sign of value (keep magnitude, sign from original)
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_copysign(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0].abs().copysign(buf[0])
        })
        .collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}
#[cfg(test)]
mod tests {
    use super::*;

    fn tuple_value(image: &ImageData, array_name: &str, tuple: usize) -> f64 {
        let array = image.point_data().get_array(array_name).unwrap();
        let mut buf = [0.0f64];
        array.tuple_as_f64(tuple, &mut buf);
        buf[0]
    }

    #[test]
    fn test() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_copysign(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn keeps_input_magnitude_and_sign() {
        let mut img = ImageData::with_dimensions(4, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![2.5, -3.5, 0.0, -0.0],
                1,
            )));

        let r = image_copysign(&img, "v");
        assert_eq!(tuple_value(&r, "v", 0), 2.5);
        assert_eq!(tuple_value(&r, "v", 1), -3.5);
        assert_eq!(tuple_value(&r, "v", 2), 0.0);
        assert_eq!(tuple_value(&r, "v", 3).to_bits(), (-0.0f64).to_bits());
    }
}
