//! Standardize to zero mean and unit variance
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_standardize(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    if n > 0 {
        let mean = values.iter().sum::<f64>() / n as f64;
        let variance = values
            .iter()
            .map(|value| {
                let delta = value - mean;
                delta * delta
            })
            .sum::<f64>()
            / n as f64;
        let stddev = variance.sqrt();
        if stddev > 0.0 {
            for value in &mut values {
                *value = (*value - mean) / stddev;
            }
        } else {
            values.fill(0.0);
        }
    }
    let dims = input.dimensions();
    let mut output = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, values, 1)));
    output.set_extent(input.extent());
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
        let r = image_standardize(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn standardizes_to_zero_mean_unit_variance() {
        let img = ImageData::from_function(
            [3, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_standardize(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        let expected = [-1.224744871391589, 0.0, 1.224744871391589];
        for (i, expected) in expected.iter().enumerate() {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - expected).abs() < 1e-12);
        }
    }
}
