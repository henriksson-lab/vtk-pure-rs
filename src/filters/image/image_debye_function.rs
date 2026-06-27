//! Debye heat capacity function
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_debye_function(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0].abs() > 0.01 {
                3.0 * (buf[0].powi(3) / ((buf[0]).exp() - 1.0).max(1e-30))
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

    fn assert_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (a, e) in actual.iter().zip(expected) {
            assert!((a - e).abs() <= e.abs().max(1.0) * 1e-12, "{a} != {e}");
        }
    }

    #[test]
    fn computes_debye_heat_capacity_kernel_with_cutoff() {
        let img = image(&[-1.0, 0.0, 0.01, 2.0]);
        let r = image_debye_function(&img, "v");
        assert_eq!(r.dimensions(), [4, 1, 1]);
        assert_eq!(r.spacing(), img.spacing());
        assert_eq!(r.origin(), img.origin());
        assert_close(
            &r.point_data().get_array("v").unwrap().to_f64_vec(),
            &[
                3.0 * (-1.0f64).powi(3) / ((-1.0f64).exp() - 1.0).max(1e-30),
                0.0,
                0.0,
                3.0 * 2.0f64.powi(3) / (2.0f64.exp() - 1.0).max(1e-30),
            ],
        );
    }
}
