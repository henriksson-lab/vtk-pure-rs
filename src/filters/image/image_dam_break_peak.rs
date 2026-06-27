//! Dam break peak discharge
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_dam_break_peak(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            8.0 / 27.0 * (9.81 * buf[0].max(0.0).powi(3)).sqrt()
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
        let r = image_dam_break_peak(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn clamps_negative_head_before_sqrt() {
        let img = ImageData::with_dimensions(3, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![-1.0, 0.0, 2.0], 1),
        ));

        let r = image_dam_break_peak(&img, "v");
        let values = r.point_data().get_array("v").unwrap().to_f64_vec();

        assert_eq!(values[0], 0.0);
        assert_eq!(values[1], 0.0);
        assert!((values[2] - (8.0 / 27.0 * (9.81f64 * 2.0f64.powi(3)).sqrt())).abs() < 1e-12);
    }
}
