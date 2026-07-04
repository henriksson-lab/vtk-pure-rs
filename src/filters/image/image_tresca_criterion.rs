//! Tresca yield criterion
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_tresca_criterion(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if matches!(a.num_components(), 3 | 6) => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let components = arr.num_components();
    let mut buf = [0.0f64; 6];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let principal_stresses = match components {
                3 => {
                    if buf[0].is_nan() {
                        return f64::NAN;
                    }
                    symmetric_eigenvalues(buf[0], buf[1], 0.0, buf[2], 0.0, 0.0)
                }
                6 => {
                    if buf[0].is_nan() {
                        return f64::NAN;
                    }
                    symmetric_eigenvalues(buf[0], buf[1], buf[2], buf[3], buf[4], buf[5])
                }
                _ => unreachable!(),
            };
            (principal_stresses[2] - principal_stresses[0]).abs()
        })
        .collect();
    let dims = input.dimensions();
    let mut output = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)));
    output.set_extent(input.extent());
    output
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
            .with_spacing([1.0, 1.0, 1.0])
            .with_origin([0.0, 0.0, 0.0])
            .with_point_array(AnyDataArray::F64(DataArray::from_vec(
                "v", values, components,
            )))
    }

    fn output_values(image: &ImageData) -> Vec<f64> {
        image.point_data().get_array("v").unwrap().to_f64_vec()
    }

    #[test]
    fn scalar_array_is_skipped_like_vtk_yield_criteria() {
        let img = image(vec![3.0, -4.0], 1);
        let r = image_tresca_criterion(&img, "v");
        assert_eq!(r.dimensions(), [2, 1, 1]);
        assert_eq!(output_values(&r), vec![3.0, -4.0]);
    }

    #[test]
    fn computes_2d_symmetric_tensor_tresca() {
        let img = image(vec![3.0, 1.0, 0.0, 1.0, 1.0, 2.0], 3);
        let r = image_tresca_criterion(&img, "v");
        let values = output_values(&r);
        assert!((values[0] - 3.0).abs() <= 1e-12);
        assert!((values[1] - 4.0).abs() <= 1e-12);
    }

    #[test]
    fn computes_3d_symmetric_tensor_tresca() {
        let img = image(vec![3.0, 2.0, -1.0, 0.0, 0.0, 0.0], 6);
        let r = image_tresca_criterion(&img, "v");
        assert_eq!(output_values(&r), vec![4.0]);
    }
}
