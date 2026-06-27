//! Dodgson quadratic interpolation
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_dodgson_quadratic(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            {
                let t = buf[0].abs();
                if t <= 0.5 {
                    0.75 - t * t
                } else if t <= 1.5 {
                    0.5 * (t - 1.5).powi(2)
                } else {
                    0.0
                }
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
        let r = image_dodgson_quadratic(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn kernel_values_are_continuous() {
        let img = ImageData::with_dimensions(4, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![0.0, 0.5, 1.5, 2.0], 1),
        ));
        let r = image_dodgson_quadratic(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];

        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.75).abs() < 1e-12);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-12);
        arr.tuple_as_f64(2, &mut buf);
        assert!(buf[0].abs() < 1e-12);
        arr.tuple_as_f64(3, &mut buf);
        assert!(buf[0].abs() < 1e-12);
    }
}
