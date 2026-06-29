//! Luminance from RGB input
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_luminance(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
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
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let img = ImageData::with_dimensions(2, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("rgb", vec![10.0, 20.0, 30.0, 100.0, 50.0, 0.0], 3),
        ));
        let r = image_luminance(&img, "rgb");
        assert_eq!(r.dimensions(), [2, 1, 1]);
        let arr = r.point_data().get_array("rgb").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 18.1).abs() < 1e-12);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 59.5).abs() < 1e-12);
    }
}
