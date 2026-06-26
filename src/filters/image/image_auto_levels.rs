//! Auto-level stretch
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_auto_levels(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    if n == 0 {
        return input.clone();
    }
    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let min_value = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_value = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max_value - min_value;
    let data: Vec<f64> = if range.abs() > 1e-15 {
        values
            .iter()
            .map(|value| (value - min_value) / range)
            .collect()
    } else {
        vec![0.0; n]
    };
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
        let r = image_auto_levels(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-12);
        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-12);
    }
}
