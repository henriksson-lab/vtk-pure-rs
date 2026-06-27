//! Hohmann transfer delta-v
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_hohmann_delta_v(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let r1 = buf[0].abs().max(1.0);
            let r2 = 2.0 * r1;
            (6.674e-11 * 5.972e24 / r1).sqrt() * ((2.0 * r2 / (r1 + r2)).sqrt() - 1.0)
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
        let r = image_hohmann_delta_v(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn clamps_radius_before_transfer_term() {
        let img = ImageData::with_dimensions(2, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![0.0, -4.0], 1),
        ));

        let r = image_hohmann_delta_v(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();

        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0].is_finite());

        arr.tuple_as_f64(1, &mut buf);
        let r1 = 4.0f64;
        let r2 = 2.0 * r1;
        let expected = (6.674e-11 * 5.972e24 / r1).sqrt() * ((2.0 * r2 / (r1 + r2)).sqrt() - 1.0);
        assert!((buf[0] - expected).abs() <= expected.abs().max(1.0) * 1e-12);
    }
}
