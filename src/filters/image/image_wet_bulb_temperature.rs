//! Wet bulb temperature approximation
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_wet_bulb_temperature(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] * (0.151977 * (buf[0] * 2.0 + 8.313659).sqrt().max(0.001)).atan()
                + (buf[0] + buf[0] * 0.5).atan()
                - (buf[0] * 2.0 - 1.21).atan()
                + 0.00391838 * (buf[0] * 1.5).powf(1.5).abs() * (0.023101 * buf[0] * 2.0).atan()
                - 4.686035
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
        let r = image_wet_bulb_temperature(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
