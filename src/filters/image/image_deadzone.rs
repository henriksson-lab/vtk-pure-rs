//! Deadzone filter (zero near zero)
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_deadzone(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0].abs() < 0.1 {
                0.0
            } else {
                buf[0]
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

    fn image(values: &[f64]) -> ImageData {
        ImageData::with_dimensions(values.len(), 1, 1)
            .with_spacing([0.5, 2.0, 1.0])
            .with_origin([1.0, -1.0, 0.0])
            .with_point_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                values.to_vec(),
                1,
            )))
    }

    #[test]
    fn zeroes_values_inside_deadzone_only() {
        let img = image(&[-0.2, -0.05, 0.1, 0.2]);
        let r = image_deadzone(&img, "v");
        assert_eq!(r.dimensions(), [4, 1, 1]);
        assert_eq!(r.spacing(), img.spacing());
        assert_eq!(r.origin(), img.origin());
        assert_eq!(
            r.point_data().get_array("v").unwrap().to_f64_vec(),
            vec![-0.2, 0.0, 0.1, 0.2]
        );
    }
}
