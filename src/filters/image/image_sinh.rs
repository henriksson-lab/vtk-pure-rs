//! Hyperbolic sine
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_sinh(input: &ImageData, scalars: &str) -> ImageData {
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
        data.extend(buf.iter().map(|value| value.sinh()));
    }
    let dims = input.dimensions();
    let mut output = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            scalars,
            data,
            num_components,
        )));
    output.set_extent(input.extent());
    output
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
        let r = image_sinh(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn test_sinh_is_not_clamped() {
        let img = ImageData::with_dimensions(1, 1, 1)
            .with_point_array(AnyDataArray::F64(DataArray::from_vec("v", vec![80.0], 1)));

        let r = image_sinh(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut tuple = [0.0];
        arr.tuple_as_f64(0, &mut tuple);
        assert_eq!(tuple[0], 80.0f64.sinh());
        assert!(tuple[0] > 1e30);
    }

    #[test]
    fn test_sinh_multi_component() {
        let img = ImageData::with_dimensions(2, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![0.0, 1.0, 2.0, 3.0], 2),
        ));

        let r = image_sinh(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut tuple = [0.0; 2];
        arr.tuple_as_f64(0, &mut tuple);
        assert_eq!(arr.num_components(), 2);
        assert_eq!(tuple[0], 0.0f64.sinh());
        assert_eq!(tuple[1], 1.0f64.sinh());
        arr.tuple_as_f64(1, &mut tuple);
        assert_eq!(tuple[0], 2.0f64.sinh());
        assert_eq!(tuple[1], 3.0f64.sinh());
    }
}
