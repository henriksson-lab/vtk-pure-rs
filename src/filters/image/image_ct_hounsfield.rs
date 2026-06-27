//! CT Hounsfield unit windowing
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_ct_hounsfield(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            ((buf[0] + 1000.0) / 2000.0).clamp(0.0, 1.0)
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
    #[test]
    fn test() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_ct_hounsfield(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn clamps_hounsfield_window() {
        let img = ImageData::with_dimensions(4, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![-1500.0, -1000.0, 0.0, 1500.0], 1),
        ));

        let r = image_ct_hounsfield(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut value = [0.0];
        let expected = [0.0, 0.0, 0.5, 1.0];
        for (i, expected_value) in expected.into_iter().enumerate() {
            arr.tuple_as_f64(i, &mut value);
            assert!((value[0] - expected_value).abs() < 1e-12);
        }
    }
}
