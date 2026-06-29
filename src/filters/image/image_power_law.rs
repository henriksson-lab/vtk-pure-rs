//! Power law x^(-2.5)
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_power_law(input: &ImageData, scalars: &str) -> ImageData {
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
                buf[0].abs().powf(-2.5) * buf[0].signum()
            } else {
                0.0
            }
        })
        .collect();
    let mut output = input.clone();
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)));
    output
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
        let r = image_power_law(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
