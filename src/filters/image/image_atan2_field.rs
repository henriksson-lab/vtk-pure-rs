//! Atan2 of two-component field to angle
use crate::data::{AnyDataArray, DataArray, ImageData};
/// Atan2 of two-component field to angle
pub fn image_atan2_field(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 2 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 2];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[1].atan2(buf[0])
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
    fn test_image_atan2_field() {
        let img = ImageData::with_dimensions(2, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![1.0, 0.0, 0.0, 1.0], 2),
        ));
        let r = image_atan2_field(&img, "v");
        assert_eq!(r.dimensions(), [2, 1, 1]);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    }
}
