//! GELU exact
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_gaussian_error_linear(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let x = buf[0];
            let z = x * std::f64::consts::FRAC_1_SQRT_2;
            let t = 1.0 / (1.0 + 0.3275911 * z.abs());
            let p = t
                * (0.254829592
                    + t * (-0.284496736
                        + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
            let erf = (1.0 - p * (-z * z).exp()) * z.signum();
            0.5 * x * (1.0 + erf)
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
        let r = image_gaussian_error_linear(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
        let arr = r.point_data().get_array("v").unwrap();
        let mut tuple = [0.0];
        arr.tuple_as_f64(0, &mut tuple);
        assert!((tuple[0] - 0.8413447361676363).abs() <= 1e-12);
    }
}
