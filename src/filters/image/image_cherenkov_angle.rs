//! Cherenkov angle
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_cherenkov_angle(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if 1.33 * buf[0] > 1.0 {
                (1.0 / (1.33 * buf[0])).clamp(-1.0, 1.0).acos()
            } else {
                0.0
            }
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
        let r = image_cherenkov_angle(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn uses_refractive_index_threshold() {
        let img = ImageData::from_function(
            [1, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.8,
        );

        let r = image_cherenkov_angle(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut value = [0.0];
        arr.tuple_as_f64(0, &mut value);

        let expected = (1.0f64 / (1.33 * 0.8)).acos();
        assert!((value[0] - expected).abs() < 1e-12);
    }
}
