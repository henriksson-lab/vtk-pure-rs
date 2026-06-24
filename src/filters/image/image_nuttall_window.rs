//! Nuttall window
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_nuttall_window(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            0.3635819 - 0.4891775 * (buf[0] * std::f64::consts::PI).cos()
                + 0.1365995 * (2.0 * buf[0] * std::f64::consts::PI).cos()
                - 0.0106411 * (3.0 * buf[0] * std::f64::consts::PI).cos()
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
        let r = image_nuttall_window(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
