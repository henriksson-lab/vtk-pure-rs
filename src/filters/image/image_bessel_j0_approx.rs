//! Bessel J0 approximation
use crate::data::{AnyDataArray, DataArray, ImageData};

fn bessel_j0_approx(x: f64) -> f64 {
    let ax = x.abs();
    if ax < 1e-8 {
        return 1.0;
    }
    if ax > 30.0 {
        return (2.0 / (std::f64::consts::PI * ax)).sqrt()
            * (ax - std::f64::consts::FRAC_PI_4).cos();
    }

    let mut term = 1.0;
    let mut sum = 1.0;
    let quarter_x2 = 0.25 * ax * ax;
    for k in 1..64 {
        let k = k as f64;
        term *= -quarter_x2 / (k * k);
        sum += term;
        if term.abs() <= sum.abs().max(1.0) * 1e-15 {
            break;
        }
    }
    sum
}

pub fn image_bessel_j0_approx(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            bessel_j0_approx(buf[0])
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
        let r = image_bessel_j0_approx(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn matches_known_j0_values() {
        assert!((bessel_j0_approx(0.0) - 1.0).abs() < 1e-12);
        assert!((bessel_j0_approx(1.0) - 0.765_197_686_6).abs() < 1e-10);
        assert!(bessel_j0_approx(2.404_825_557_7).abs() < 1e-6);
    }
}
