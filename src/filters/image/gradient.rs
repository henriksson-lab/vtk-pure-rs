use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the gradient of a scalar field on ImageData using central differences.
///
/// Adds a "Gradient" 3-component point data array containing [dF/dx, dF/dy, dF/dz]
/// at each grid point. Optionally also adds a "GradientMagnitude" scalar array.
pub fn image_gradient(input: &ImageData, scalars: &str, compute_magnitude: bool) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let spacing = input.spacing();
    let n = nx * ny * nz;

    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let idx = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };

    let mut grad = vec![0.0f64; n * 3];
    let mut mag = if compute_magnitude {
        vec![0.0f64; n]
    } else {
        vec![]
    };

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let pi = idx(i, j, k);

                // Central differences with clamped boundaries
                let im = if i > 0 { i - 1 } else { 0 };
                let ip = if i + 1 < nx { i + 1 } else { nx - 1 };
                let jm = if j > 0 { j - 1 } else { 0 };
                let jp = if j + 1 < ny { j + 1 } else { ny - 1 };
                let km = if k > 0 { k - 1 } else { 0 };
                let kp = if k + 1 < nz { k + 1 } else { nz - 1 };

                let dx_span = (ip - im) as f64 * spacing[0];
                let dy_span = (jp - jm) as f64 * spacing[1];
                let dz_span = (kp - km) as f64 * spacing[2];

                let gx = if dx_span > 1e-15 {
                    (values[idx(ip, j, k)] - values[idx(im, j, k)]) / dx_span
                } else {
                    0.0
                };
                let gy = if dy_span > 1e-15 {
                    (values[idx(i, jp, k)] - values[idx(i, jm, k)]) / dy_span
                } else {
                    0.0
                };
                let gz = if dz_span > 1e-15 {
                    (values[idx(i, j, kp)] - values[idx(i, j, km)]) / dz_span
                } else {
                    0.0
                };

                grad[pi * 3] = gx;
                grad[pi * 3 + 1] = gy;
                grad[pi * 3 + 2] = gz;

                if compute_magnitude {
                    mag[pi] = (gx * gx + gy * gy + gz * gz).sqrt();
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Gradient", grad, 3)));
    if compute_magnitude {
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "GradientMagnitude",
                mag,
                1,
            )));
    }
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_gradient_x() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        // f(x) = x: gradient should be [1, 0, 0]
        let values: Vec<f64> = (0..5).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", values, 1)));

        let result = image_gradient(&img, "val", true);
        let grad = result.point_data().get_array("Gradient").unwrap();
        let mut buf = [0.0f64; 3];
        // Interior point
        grad.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
        assert!(buf[1].abs() < 1e-10);
        assert!(buf[2].abs() < 1e-10);
    }

    #[test]
    fn gradient_magnitude() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        // f = x + y
        let mut values = Vec::new();
        for j in 0..5 {
            for i in 0..5 {
                values.push(i as f64 + j as f64);
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", values, 1)));

        let result = image_gradient(&img, "val", true);
        let mag = result.point_data().get_array("GradientMagnitude").unwrap();
        let mut buf = [0.0f64];
        // Interior point: gradient = (1,1,0), magnitude = sqrt(2)
        mag.tuple_as_f64(12, &mut buf); // (2,2,0)
        assert!((buf[0] - 2.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn missing_scalars() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let result = image_gradient(&img, "nope", false);
        assert!(result.point_data().get_array("Gradient").is_none());
    }
}
