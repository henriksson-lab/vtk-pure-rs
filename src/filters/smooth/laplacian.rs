use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the Laplacian of a scalar field on ImageData.
///
/// Uses the standard 7-point stencil (6 neighbors + center) for the
/// discrete Laplacian: ∇²f ≈ Σ(f_neighbor - f_center) / h².
/// Adds a "Laplacian" scalar array.
pub fn image_laplacian(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let sp = input.spacing();
    let n = nx * ny * nz;

    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let idx = |i: usize, j: usize, k: usize| k * ny * nx + j * nx + i;
    let hx2 = sp[0] * sp[0];
    let hy2 = sp[1] * sp[1];
    let hz2 = sp[2] * sp[2];

    let mut lapl = vec![0.0f64; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let c = values[idx(i, j, k)];
                let im = if i > 0 { values[idx(i - 1, j, k)] } else { c };
                let ip = if i + 1 < nx {
                    values[idx(i + 1, j, k)]
                } else {
                    c
                };
                let jm = if j > 0 { values[idx(i, j - 1, k)] } else { c };
                let jp = if j + 1 < ny {
                    values[idx(i, j + 1, k)]
                } else {
                    c
                };
                let km = if k > 0 { values[idx(i, j, k - 1)] } else { c };
                let kp = if k + 1 < nz {
                    values[idx(i, j, k + 1)]
                } else {
                    c
                };

                lapl[idx(i, j, k)] = (ip - 2.0 * c + im) / hx2
                    + (jp - 2.0 * c + jm) / hy2
                    + (kp - 2.0 * c + km) / hz2;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Laplacian", lapl, 1)));
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
