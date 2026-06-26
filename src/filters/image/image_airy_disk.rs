//! Airy disk pattern (2*J1(x)/x)^2 approx
use crate::data::{AnyDataArray, DataArray, ImageData};

fn bessel_j1_approx(x: f64) -> f64 {
    let ax = x.abs();
    if ax < 1e-8 {
        return 0.5 * x;
    }

    let mut term = 0.5 * ax;
    let mut sum = term;
    let quarter_x2 = 0.25 * ax * ax;
    for k in 1..64 {
        let k = k as f64;
        term *= -quarter_x2 / (k * (k + 1.0));
        sum += term;
        if term.abs() <= sum.abs().max(1.0) * 1e-15 {
            break;
        }
    }

    sum.copysign(x)
}

pub fn image_airy_disk(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            {
                let x = buf[0].abs() * std::f64::consts::PI;
                if x < 1e-8 {
                    1.0
                } else {
                    let j1 = bessel_j1_approx(x);
                    (2.0 * j1 / x).powi(2).clamp(0.0, 1.0)
                }
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
        let r = image_airy_disk(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn unit_intensity_at_origin() {
        let img = ImageData::from_function(
            [1, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.0,
        );
        let r = image_airy_disk(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-12);
    }
}
