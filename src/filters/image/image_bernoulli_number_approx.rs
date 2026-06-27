//! Bernoulli number approximation
use crate::data::{AnyDataArray, DataArray, ImageData};

fn bernoulli_number_approx(x: f64) -> f64 {
    let n = x.abs().round().max(1.0) as u32;
    if n == 1 {
        return -0.5;
    }
    if n % 2 == 1 {
        return 0.0;
    }

    let factorial = (2..=n).fold(1.0, |acc, k| acc * k as f64);
    let magnitude = 2.0 * factorial / (2.0 * std::f64::consts::PI).powi(n as i32);
    if (n / 2) % 2 == 0 {
        magnitude
    } else {
        -magnitude
    }
}

pub fn image_bernoulli_number_approx(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            bernoulli_number_approx(buf[0])
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
        let r = image_bernoulli_number_approx(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn preserves_basic_bernoulli_number_behavior() {
        assert!((bernoulli_number_approx(1.0) + 0.5).abs() < 1e-12);
        assert!((bernoulli_number_approx(2.0) - 1.0 / 6.0).abs() < 0.01);
        assert_eq!(bernoulli_number_approx(3.0), 0.0);
        assert!((bernoulli_number_approx(4.0) + 1.0 / 30.0).abs() < 0.005);
    }
}
