//! Critical density of universe
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_critical_density(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            3.0 * (buf[0] / 3.086e22).powi(2) / (8.0 * std::f64::consts::PI * 6.674e-11)
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
        let r = image_critical_density(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
        let arr = r.point_data().get_array("v").unwrap();
        let mut tuple = [0.0];
        arr.tuple_as_f64(0, &mut tuple);
        let expected = 3.0 * (1.0f64 / 3.086e22).powi(2) / (8.0 * std::f64::consts::PI * 6.674e-11);
        assert!((tuple[0] - expected).abs() < expected * 1e-12);
        arr.tuple_as_f64(4, &mut tuple);
        assert!(tuple[0] > expected);
    }
}
