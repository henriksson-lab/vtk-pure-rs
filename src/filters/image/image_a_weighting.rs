//! A-weighting frequency response
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_a_weighting(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let f2 = buf[0] * buf[0];
            1.2588966e16 * f2 * f2
                / ((f2 + 20.6f64.powi(2))
                    * (f2 + 12194.0f64.powi(2)).sqrt().powi(2)
                    * (f2 + 107.7f64.powi(2)).sqrt()
                    * (f2 + 737.9f64.powi(2)).sqrt())
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
        let r = image_a_weighting(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
