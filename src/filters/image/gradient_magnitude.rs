use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute gradient magnitude from a scalar array on ImageData using central differences.
///
/// Adds a "GradientMagnitude" point data array to the output. At each grid
/// point, the gradient is computed via duplicated-boundary central differences
/// in XY, matching vtkImageGradientMagnitude's default dimensionality.
pub fn image_gradient_magnitude(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        _ => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let spacing = input.spacing();
    let n: usize = nx * ny * nz;
    let num_components = arr.num_components();

    // Read scalar values into a flat array.
    let mut values = vec![0.0f64; n * num_components];
    let mut buf = vec![0.0f64; num_components];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let base = i * num_components;
        values[base..base + num_components].copy_from_slice(&buf);
    }

    let idx = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };

    let mut mag = vec![0.0f64; n * num_components];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let im: usize = if i > 0 { i - 1 } else { 0 };
                let ip: usize = if i + 1 < nx { i + 1 } else { nx - 1 };
                let jm: usize = if j > 0 { j - 1 } else { 0 };
                let jp: usize = if j + 1 < ny { j + 1 } else { ny - 1 };

                let pi: usize = idx(i, j, k);
                let x_min = idx(im, j, k) * num_components;
                let x_max = idx(ip, j, k) * num_components;
                let y_min = idx(i, jm, k) * num_components;
                let y_max = idx(i, jp, k) * num_components;
                let out = pi * num_components;
                for component in 0..num_components {
                    let gx: f64 = if spacing[0].abs() > 1e-15 {
                        (values[x_max + component] - values[x_min + component]) * 0.5 / spacing[0]
                    } else {
                        0.0
                    };
                    let gy: f64 = if spacing[1].abs() > 1e-15 {
                        (values[y_max + component] - values[y_min + component]) * 0.5 / spacing[1]
                    } else {
                        0.0
                    };
                    mag[out + component] = (gx * gx + gy * gy).sqrt();
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GradientMagnitude",
            mag,
            num_components,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_ramp_x() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        let values: Vec<f64> = (0..5).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("Scalars", values, 1)));

        let result = image_gradient_magnitude(&img, "Scalars");
        let arr = result.point_data().get_array("GradientMagnitude").unwrap();
        assert_eq!(arr.num_tuples(), 5);
        let mut buf = [0.0f64];
        // Interior point (index 2): gradient = (1,0,0), magnitude = 1
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn diagonal_field() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        // f = x + y => gradient = (1,1,0), magnitude = sqrt(2)
        let mut values: Vec<f64> = Vec::new();
        for j in 0..5 {
            for i in 0..5 {
                values.push(i as f64 + j as f64);
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("S", values, 1)));

        let result = image_gradient_magnitude(&img, "S");
        let arr = result.point_data().get_array("GradientMagnitude").unwrap();
        let mut buf = [0.0f64];
        // Interior point (2,2): magnitude = sqrt(2)
        arr.tuple_as_f64(12, &mut buf);
        assert!((buf[0] - 2.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn default_dimensionality_ignores_z() {
        let mut img = ImageData::with_dimensions(1, 1, 5);
        img.set_spacing([1.0, 1.0, 1.0]);
        let values: Vec<f64> = (0..5).map(|k| k as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("S", values, 1)));

        let result = image_gradient_magnitude(&img, "S");
        let arr = result.point_data().get_array("GradientMagnitude").unwrap();
        let mut buf = [1.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn missing_scalars_returns_clone() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let result = image_gradient_magnitude(&img, "nonexistent");
        assert!(result.point_data().get_array("GradientMagnitude").is_none());
    }

    #[test]
    fn multi_component_field_preserves_components() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "uv",
                vec![0.0, 0.0, 1.0, 2.0, 2.0, 4.0],
                2,
            )));

        let result = image_gradient_magnitude(&img, "uv");
        let arr = result.point_data().get_array("GradientMagnitude").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
        assert!((buf[1] - 2.0).abs() < 1e-10);
    }
}
