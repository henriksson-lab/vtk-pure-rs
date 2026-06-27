//! Diffusion tensor fractional anisotropy
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_diffusion_tensor_fa(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if matches!(a.num_components(), 1 | 6 | 9) => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let components = arr.num_components();
    let mut buf = [0.0f64; 9];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let eigenvalues = match components {
                1 => [buf[0], buf[0], buf[0]],
                6 => symmetric_eigenvalues(buf[0], buf[1], buf[2], buf[3], buf[4], buf[5]),
                9 => symmetric_eigenvalues(
                    buf[0],
                    buf[4],
                    buf[8],
                    0.5 * (buf[1] + buf[3]),
                    0.5 * (buf[5] + buf[7]),
                    0.5 * (buf[2] + buf[6]),
                ),
                _ => unreachable!(),
            };
            fractional_anisotropy(eigenvalues)
        })
        .collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

fn fractional_anisotropy(eigenvalues: [f64; 3]) -> f64 {
    let mean = (eigenvalues[0] + eigenvalues[1] + eigenvalues[2]) / 3.0;
    let numerator = 1.5
        * ((eigenvalues[0] - mean).powi(2)
            + (eigenvalues[1] - mean).powi(2)
            + (eigenvalues[2] - mean).powi(2));
    let denominator = eigenvalues[0].powi(2) + eigenvalues[1].powi(2) + eigenvalues[2].powi(2);
    if denominator <= 1e-30 {
        0.0
    } else {
        (numerator / denominator).sqrt()
    }
}

fn symmetric_eigenvalues(a11: f64, a22: f64, a33: f64, a12: f64, a23: f64, a13: f64) -> [f64; 3] {
    let p1 = a12 * a12 + a13 * a13 + a23 * a23;
    if p1.abs() < 1e-30 {
        let mut vals = [a11, a22, a33];
        vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        return vals;
    }

    let q = (a11 + a22 + a33) / 3.0;
    let p2 = (a11 - q).powi(2) + (a22 - q).powi(2) + (a33 - q).powi(2) + 2.0 * p1;
    let p = (p2 / 6.0).sqrt();

    let b11 = (a11 - q) / p;
    let b22 = (a22 - q) / p;
    let b33 = (a33 - q) / p;
    let b12 = a12 / p;
    let b13 = a13 / p;
    let b23 = a23 / p;

    let det_b = b11 * (b22 * b33 - b23 * b23) - b12 * (b12 * b33 - b23 * b13)
        + b13 * (b12 * b23 - b22 * b13);
    let r = (det_b / 2.0).clamp(-1.0, 1.0);

    let phi = if r <= -1.0 {
        std::f64::consts::PI / 3.0
    } else if r >= 1.0 {
        0.0
    } else {
        r.acos() / 3.0
    };

    let eig1 = q + 2.0 * p * phi.cos();
    let eig3 = q + 2.0 * p * (phi + 2.0 * std::f64::consts::PI / 3.0).cos();
    let eig2 = 3.0 * q - eig1 - eig3;

    let mut vals = [eig1, eig2, eig3];
    vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    vals
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(values: Vec<f64>, components: usize) -> ImageData {
        ImageData::with_dimensions(values.len() / components, 1, 1)
            .with_spacing([0.5, 2.0, 1.0])
            .with_origin([1.0, -1.0, 0.0])
            .with_point_array(AnyDataArray::F64(DataArray::from_vec(
                "v", values, components,
            )))
    }

    fn assert_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (a, e) in actual.iter().zip(expected) {
            assert!((a - e).abs() <= 1e-12, "{a} != {e}");
        }
    }

    #[test]
    fn isotropic_scalar_has_zero_fractional_anisotropy() {
        let img = image(vec![2.0, 0.0], 1);
        let r = image_diffusion_tensor_fa(&img, "v");
        assert_eq!(r.dimensions(), [2, 1, 1]);
        assert_eq!(r.spacing(), img.spacing());
        assert_eq!(r.origin(), img.origin());
        assert_close(
            &r.point_data().get_array("v").unwrap().to_f64_vec(),
            &[0.0, 0.0],
        );
    }

    #[test]
    fn computes_diagonal_tensor_fractional_anisotropy() {
        let img = image(vec![3.0, 2.0, 1.0, 0.0, 0.0, 0.0], 6);
        let r = image_diffusion_tensor_fa(&img, "v");
        assert_close(
            &r.point_data().get_array("v").unwrap().to_f64_vec(),
            &[((1.5 * (1.0 + 0.0 + 1.0)) / 14.0_f64).sqrt()],
        );
    }

    #[test]
    fn computes_full_tensor_fractional_anisotropy() {
        let img = image(vec![3.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 1.0], 9);
        let r = image_diffusion_tensor_fa(&img, "v");
        assert_close(
            &r.point_data().get_array("v").unwrap().to_f64_vec(),
            &[((1.5 * (1.0 + 0.0 + 1.0)) / 14.0_f64).sqrt()],
        );
    }
}
