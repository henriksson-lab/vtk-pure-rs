//! Beaufort wind scale
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_beaufort_scale(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            (buf[0].abs() / 0.836)
                .powf(2.0 / 3.0)
                .round()
                .clamp(0.0, 12.0)
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
        let r = image_beaufort_scale(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn uses_wind_speed_magnitude() {
        let img = ImageData::from_function(
            [1, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| -0.836,
        );
        let r = image_beaufort_scale(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut tuple = [0.0];
        arr.tuple_as_f64(0, &mut tuple);
        assert_eq!(tuple[0], 1.0);
    }
}
