//! Bernoulli number approximation
use crate::data::{AnyDataArray, DataArray, ImageData};

/// Riemann zeta function for real `s >= 2`, by Euler-Maclaurin summation.
///
/// The naive series `sum k^-s` converges far too slowly at small `s` (it needs
/// ~1e6 terms for six digits at `s = 2`), so the tail beyond `TERMS` is replaced
/// by its integral plus the first two Euler-Maclaurin corrections.
fn zeta(s: f64) -> f64 {
    const TERMS: u32 = 16;
    let mut sum: f64 = (1..TERMS).map(|k| (k as f64).powf(-s)).sum();
    let n = TERMS as f64;
    sum += 0.5 * n.powf(-s);
    sum += n.powf(1.0 - s) / (s - 1.0);
    sum += s / 12.0 * n.powf(-s - 1.0);
    sum -= s * (s + 1.0) * (s + 2.0) / 720.0 * n.powf(-s - 3.0);
    sum
}

/// Bernoulli number `B_n` for `n = round(|x|)`, via Euler's closed form
///
/// ```text
/// B_n = (-1)^(n/2 + 1) * 2 * n! / (2*pi)^n * zeta(n)     (n even, n >= 2)
/// ```
///
/// with `B_1 = -1/2` and `B_n = 0` for odd `n > 1`. Dropping the `zeta(n)`
/// factor would be a 65% error at `n = 2`, and the sign alternates starting
/// *positive* at `B_2 = +1/6` (`B_4 = -1/30`, `B_6 = +1/42`, ...).
///
/// `n!/(2*pi)^n` is accumulated as a running product of `k/(2*pi)` so the
/// intermediate factorial does not overflow before the result itself does.
fn bernoulli_number_approx(x: f64) -> f64 {
    let n = x.abs().round().max(1.0) as u32;
    if n == 1 {
        return -0.5;
    }
    if n % 2 == 1 {
        return 0.0;
    }

    let mut magnitude = 2.0;
    for k in 1..=n {
        magnitude *= k as f64 / std::f64::consts::TAU;
    }
    magnitude *= zeta(n as f64);

    if (n / 2) % 2 == 1 {
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
