use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the divergence of a vector field on ImageData.
///
/// Takes a 3-component array (specified by `vector_array`) and produces a
/// scalar "Divergence" point data array using central finite differences:
///   div(V) = dVx/dx + dVy/dy + dVz/dz
pub fn compute_divergence(input: &ImageData, vector_array: &str) -> ImageData {
    let arr = match input.point_data().get_array(vector_array) {
        Some(a) => a,
        None => return input.clone(),
    };

    let num_components = arr.num_components().min(3);
    if num_components == 0 {
        return input.clone();
    }

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let spacing = input.spacing();
    let n: usize = nx * ny * nz;

    // Extract up to the first three vector components, matching vtkImageDivergence.
    let mut vx: Vec<f64> = vec![0.0; n];
    let mut vy: Vec<f64> = vec![0.0; n];
    let mut vz: Vec<f64> = vec![0.0; n];
    let mut buf: [f64; 3] = [0.0, 0.0, 0.0];
    for idx in 0..n {
        arr.tuple_as_f64(idx, &mut buf);
        vx[idx] = buf[0];
        if num_components > 1 {
            vy[idx] = buf[1];
        }
        if num_components > 2 {
            vz[idx] = buf[2];
        }
    }

    let index = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };

    let mut divergence: Vec<f64> = vec![0.0; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let im: usize = if i > 0 { i - 1 } else { 0 };
                let ip: usize = if i + 1 < nx { i + 1 } else { nx - 1 };
                let jm: usize = if j > 0 { j - 1 } else { 0 };
                let jp: usize = if j + 1 < ny { j + 1 } else { ny - 1 };
                let km: usize = if k > 0 { k - 1 } else { 0 };
                let kp: usize = if k + 1 < nz { k + 1 } else { nz - 1 };

                let dx_span: f64 = (ip - im) as f64 * spacing[0];
                let dy_span: f64 = (jp - jm) as f64 * spacing[1];
                let dz_span: f64 = (kp - km) as f64 * spacing[2];

                let dvx_dx: f64 = if dx_span > 1e-15 {
                    (vx[index(ip, j, k)] - vx[index(im, j, k)]) / dx_span
                } else {
                    0.0
                };
                let dvy_dy: f64 = if dy_span > 1e-15 {
                    (vy[index(i, jp, k)] - vy[index(i, jm, k)]) / dy_span
                } else {
                    0.0
                };
                let dvz_dz: f64 = if dz_span > 1e-15 {
                    (vz[index(i, j, kp)] - vz[index(i, j, km)]) / dz_span
                } else {
                    0.0
                };

                let mut sum = dvx_dx;
                if num_components > 1 {
                    sum += dvy_dy;
                }
                if num_components > 2 {
                    sum += dvz_dz;
                }

                divergence[index(i, j, k)] = sum;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Divergence",
            divergence,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_field_zero_divergence() {
        // Constant vector field V = (1, 2, 3) everywhere => div = 0
        let mut img = ImageData::with_dimensions(4, 4, 4);
        img.set_spacing([1.0, 1.0, 1.0]);

        let n: usize = 4 * 4 * 4;
        let mut data: Vec<f64> = Vec::with_capacity(n * 3);
        for _ in 0..n {
            data.push(1.0);
            data.push(2.0);
            data.push(3.0);
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("Velocity", data, 3)));

        let result = compute_divergence(&img, "Velocity");
        let div_arr = result.point_data().get_array("Divergence").unwrap();
        let mut buf: [f64; 1] = [0.0];
        // Check interior points (away from boundary) for zero divergence
        for k in 1..3 {
            for j in 1..3 {
                for i in 1..3 {
                    let idx: usize = k * 16 + j * 4 + i;
                    div_arr.tuple_as_f64(idx, &mut buf);
                    assert!(buf[0].abs() < 1e-10, "Expected ~0, got {}", buf[0]);
                }
            }
        }
    }

    #[test]
    fn linear_field_constant_divergence() {
        // V = (x, 0, 0) => div = 1
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.set_spacing([1.0, 1.0, 1.0]);

        let mut data: Vec<f64> = Vec::new();
        for i in 0..5 {
            data.push(i as f64); // vx = x
            data.push(0.0);
            data.push(0.0);
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("V", data, 3)));

        let result = compute_divergence(&img, "V");
        let div_arr = result.point_data().get_array("Divergence").unwrap();
        let mut buf: [f64; 1] = [0.0];

        // Interior points should have divergence = 1
        for i in 1..4 {
            div_arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - 1.0).abs() < 1e-10, "Expected 1.0, got {}", buf[0]);
        }
    }

    #[test]
    fn two_component_field_uses_x_and_y_terms() {
        // V = (x, y) => div = 2
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.set_spacing([1.0, 1.0, 1.0]);

        let mut data: Vec<f64> = Vec::new();
        for j in 0..5 {
            for i in 0..5 {
                data.push(i as f64);
                data.push(j as f64);
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("V", data, 2)));

        let result = compute_divergence(&img, "V");
        let div_arr = result.point_data().get_array("Divergence").unwrap();
        let mut buf: [f64; 1] = [0.0];
        div_arr.tuple_as_f64(2 * 5 + 2, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10, "Expected 2.0, got {}", buf[0]);
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let result = compute_divergence(&img, "NonExistent");
        assert_eq!(result.dimensions(), img.dimensions());
    }
}
