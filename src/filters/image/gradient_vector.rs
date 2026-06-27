use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the gradient of a scalar field on ImageData as a 2-component vector field.
///
/// Uses duplicated-boundary central differences. Adds a "GradientVector"
/// point data array containing [dF/dx, dF/dy], matching vtkImageGradient's
/// default dimensionality.
pub fn image_gradient_vector(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let spacing = input.spacing();
    let n: usize = nx * ny * nz;
    if arr.num_tuples() != n {
        return input.clone();
    }

    // Read scalar values
    let mut values: Vec<f64> = vec![0.0; n];
    let mut buf: [f64; 1] = [0.0];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let idx = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };

    let dimensionality = 2;
    let mut grad: Vec<f64> = vec![0.0; n * dimensionality];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let pi: usize = idx(i, j, k);

                // Central differences with clamped boundaries
                let im: usize = if i > 0 { i - 1 } else { 0 };
                let ip: usize = if i + 1 < nx { i + 1 } else { nx - 1 };
                let jm: usize = if j > 0 { j - 1 } else { 0 };
                let jp: usize = if j + 1 < ny { j + 1 } else { ny - 1 };
                let gx: f64 = if spacing[0].abs() > 1e-15 {
                    (values[idx(ip, j, k)] - values[idx(im, j, k)]) * 0.5 / spacing[0]
                } else {
                    0.0
                };
                let gy: f64 = if spacing[1].abs() > 1e-15 {
                    (values[idx(i, jp, k)] - values[idx(i, jm, k)]) * 0.5 / spacing[1]
                } else {
                    0.0
                };

                grad[pi * dimensionality] = gx;
                grad[pi * dimensionality + 1] = gy;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GradientVector",
            grad,
            dimensionality,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image(nx: usize, ny: usize, nz: usize, values: Vec<f64>) -> ImageData {
        let mut img = ImageData::with_dimensions(nx, ny, nz);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("Scalars", values, 1)));
        img
    }

    #[test]
    fn constant_field_zero_gradient() {
        let n: usize = 3 * 3 * 3;
        let img = make_image(3, 3, 3, vec![5.0; n]);
        let result = image_gradient_vector(&img, "Scalars");
        let grad = result.point_data().get_array("GradientVector").unwrap();
        assert_eq!(grad.num_components(), 2);
        let mut buf: [f64; 2] = [0.0; 2];
        for i in 0..n {
            grad.tuple_as_f64(i, &mut buf);
            assert!(buf[0].abs() < 1e-10);
            assert!(buf[1].abs() < 1e-10);
        }
    }

    #[test]
    fn linear_x_gradient() {
        // 5x1x1 image with values [0, 1, 2, 3, 4], spacing 1.0
        let img = make_image(5, 1, 1, vec![0.0, 1.0, 2.0, 3.0, 4.0]);
        let result = image_gradient_vector(&img, "Scalars");
        let grad = result.point_data().get_array("GradientVector").unwrap();
        assert_eq!(grad.num_components(), 2);
        let mut buf: [f64; 2] = [0.0; 2];
        // Interior point (index 2): central diff = (3 - 1) / 2 = 1.0
        grad.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10, "gx = {}", buf[0]);
        assert!(buf[1].abs() < 1e-10);
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = ImageData::with_dimensions(2, 2, 2);
        let result = image_gradient_vector(&img, "NonExistent");
        assert_eq!(result.dimensions(), [2, 2, 2]);
    }
}
