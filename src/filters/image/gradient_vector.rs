use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the gradient of a scalar field on ImageData as a 3-component vector field.
///
/// Uses central differences. Adds a "GradientVector" 3-component point data array
/// containing [dF/dx, dF/dy, dF/dz] at each grid point.
pub fn image_gradient_vector(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let spacing = input.spacing();
    let n: usize = nx * ny * nz;

    // Read scalar values
    let mut values: Vec<f64> = vec![0.0; n];
    let mut buf: [f64; 1] = [0.0];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let idx = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };

    let mut grad: Vec<f64> = vec![0.0; n * 3];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let pi: usize = idx(i, j, k);

                // Central differences with clamped boundaries
                let im: usize = if i > 0 { i - 1 } else { 0 };
                let ip: usize = if i + 1 < nx { i + 1 } else { nx - 1 };
                let jm: usize = if j > 0 { j - 1 } else { 0 };
                let jp: usize = if j + 1 < ny { j + 1 } else { ny - 1 };
                let km: usize = if k > 0 { k - 1 } else { 0 };
                let kp: usize = if k + 1 < nz { k + 1 } else { nz - 1 };

                let dx_span: f64 = (ip - im) as f64 * spacing[0];
                let dy_span: f64 = (jp - jm) as f64 * spacing[1];
                let dz_span: f64 = (kp - km) as f64 * spacing[2];

                let gx: f64 = if dx_span > 1e-15 {
                    (values[idx(ip, j, k)] - values[idx(im, j, k)]) / dx_span
                } else {
                    0.0
                };
                let gy: f64 = if dy_span > 1e-15 {
                    (values[idx(i, jp, k)] - values[idx(i, jm, k)]) / dy_span
                } else {
                    0.0
                };
                let gz: f64 = if dz_span > 1e-15 {
                    (values[idx(i, j, kp)] - values[idx(i, j, km)]) / dz_span
                } else {
                    0.0
                };

                grad[pi * 3] = gx;
                grad[pi * 3 + 1] = gy;
                grad[pi * 3 + 2] = gz;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GradientVector",
            grad,
            3,
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
        let mut buf: [f64; 3] = [0.0; 3];
        for i in 0..n {
            grad.tuple_as_f64(i, &mut buf);
            assert!(buf[0].abs() < 1e-10);
            assert!(buf[1].abs() < 1e-10);
            assert!(buf[2].abs() < 1e-10);
        }
    }

    #[test]
    fn linear_x_gradient() {
        // 5x1x1 image with values [0, 1, 2, 3, 4], spacing 1.0
        let img = make_image(5, 1, 1, vec![0.0, 1.0, 2.0, 3.0, 4.0]);
        let result = image_gradient_vector(&img, "Scalars");
        let grad = result.point_data().get_array("GradientVector").unwrap();
        let mut buf: [f64; 3] = [0.0; 3];
        // Interior point (index 2): central diff = (3 - 1) / 2 = 1.0
        grad.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10, "gx = {}", buf[0]);
        assert!(buf[1].abs() < 1e-10);
        assert!(buf[2].abs() < 1e-10);
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = ImageData::with_dimensions(2, 2, 2);
        let result = image_gradient_vector(&img, "NonExistent");
        assert_eq!(result.dimensions(), [2, 2, 2]);
    }
}
