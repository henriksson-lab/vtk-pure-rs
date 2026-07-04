use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the discrete Laplacian of a scalar field on ImageData.
///
/// Like vtkImageLaplacian, this computes an XY Laplacian by default.
/// Boundary pixels are replicated.
///
/// Adds a "Laplacian" point data array.
pub fn image_laplacian(input: &ImageData, scalars: &str) -> ImageData {
    image_laplacian_with_dimensionality(input, scalars, 2)
}

/// Compute vtkImageLaplacian with explicit dimensionality clamped to 2 or 3.
pub fn image_laplacian_with_dimensionality(
    input: &ImageData,
    scalars: &str,
    dimensionality: usize,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let num_components = arr.num_components();

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let spacing = input.spacing();
    let n: usize = nx * ny * nz;

    let mut values = vec![0.0f64; n * num_components];
    let mut buf = vec![0.0f64; num_components];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i * num_components..(i + 1) * num_components].copy_from_slice(&buf);
    }

    let idx = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };

    let dx2: f64 = spacing[0] * spacing[0];
    let dy2: f64 = spacing[1] * spacing[1];
    let dz2: f64 = spacing[2] * spacing[2];

    let axes_num = dimensionality.clamp(2, 3);
    let mut laplacian = vec![0.0f64; n * num_components];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let pi = idx(i, j, k);
                let im = if i > 0 { i - 1 } else { 0 };
                let ip = if i + 1 < nx { i + 1 } else { nx - 1 };
                let jm = if j > 0 { j - 1 } else { 0 };
                let jp = if j + 1 < ny { j + 1 } else { ny - 1 };
                let km = if k > 0 { k - 1 } else { 0 };
                let kp = if k + 1 < nz { k + 1 } else { nz - 1 };

                for c in 0..num_components {
                    let value_at = |point: usize| values[point * num_components + c];
                    let center = value_at(pi);

                    let mut sum = if dx2 > 1e-30 {
                        (value_at(idx(ip, j, k)) - 2.0 * center + value_at(idx(im, j, k))) / dx2
                    } else {
                        0.0
                    };

                    if dy2 > 1e-30 {
                        sum += (value_at(idx(i, jp, k)) - 2.0 * center + value_at(idx(i, jm, k)))
                            / dy2;
                    }

                    if axes_num == 3 && dz2 > 1e-30 {
                        sum += (value_at(idx(i, j, kp)) - 2.0 * center + value_at(idx(i, j, km)))
                            / dz2;
                    }

                    laplacian[pi * num_components + c] = sum;
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Laplacian",
            laplacian,
            num_components,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadratic_field() {
        // f = x² -> Laplacian = 2 (constant)
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        let values: Vec<f64> = (0..5).map(|i| (i as f64) * (i as f64)).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("f", values, 1)));

        let result = image_laplacian(&img, "f");
        let arr = result.point_data().get_array("Laplacian").unwrap();
        let mut buf = [0.0f64];
        // Interior points should have Laplacian ≈ 2
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn constant_field_zero() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![5.0; 27],
                1,
            )));

        let result = image_laplacian(&img, "f");
        let arr = result.point_data().get_array("Laplacian").unwrap();
        let mut buf = [0.0f64];
        for i in 0..27 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0].abs() < 1e-10);
        }
    }

    #[test]
    fn missing_scalars() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let result = image_laplacian(&img, "nope");
        assert!(result.point_data().get_array("Laplacian").is_none());
    }
}
