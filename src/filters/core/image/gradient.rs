use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the gradient of a scalar field on ImageData using central differences.
///
/// Adds a "Gradient" point data array. Like vtkImageGradient, this computes a
/// two-dimensional XY gradient by default. Optionally also adds a
/// "GradientMagnitude" scalar array.
pub fn image_gradient(input: &ImageData, scalars: &str, compute_magnitude: bool) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        None => return input.clone(),
        _ => return input.clone(),
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

    let dimensionality = 2;
    let mut grad = vec![0.0f64; n * dimensionality];
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
                let gx = if spacing[0].abs() > 1e-15 {
                    (values[idx(ip, j, k)] - values[idx(im, j, k)]) * 0.5 / spacing[0]
                } else {
                    0.0
                };
                let gy = if spacing[1].abs() > 1e-15 {
                    (values[idx(i, jp, k)] - values[idx(i, jm, k)]) * 0.5 / spacing[1]
                } else {
                    0.0
                };

                grad[pi * dimensionality] = gx;
                grad[pi * dimensionality + 1] = gy;

                if compute_magnitude {
                    mag[pi] = (gx * gx + gy * gy).sqrt();
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Gradient",
            grad,
            dimensionality,
        )));
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

    #[test]
    fn multi_component_scalars_are_rejected() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "rgb",
                vec![1.0, 2.0, 3.0, 4.0],
                2,
            )));
        let result = image_gradient(&img, "rgb", false);
        assert!(result.point_data().get_array("Gradient").is_none());
    }
}
