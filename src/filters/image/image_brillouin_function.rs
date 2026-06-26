//! Brillouin function
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_brillouin_function(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let x = buf[0] * 0.01;
            let j = 3.5;
            if x.abs() < 1e-8 {
                (j + 1.0) / (3.0 * j) * x
            } else {
                let two_j = 2.0 * j;
                let coth = |v: f64| 1.0 / v.tanh();
                (two_j + 1.0) / two_j * coth((two_j + 1.0) / two_j * x)
                    - 1.0 / two_j * coth(x / two_j)
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
        let r = image_brillouin_function(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.0);
        assert!(buf[0] < 0.01);
    }
}
