use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the divergence of a vector field on ImageData.
///
/// Takes a 1- to 3-component array (specified by `vector_array`) and produces a
/// scalar "Divergence" point data array using VTK-style central differences:
///   div(V) = dVx/dx + dVy/dy + dVz/dz
pub fn compute_divergence(input: &ImageData, vector_array: &str) -> ImageData {
    let arr = match input.point_data().get_array(vector_array) {
        Some(a) => a,
        None => return input.clone(),
    };

    let max_c = arr.num_components().min(3);
    if max_c == 0 {
        return input.clone();
    }

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let spacing = input.spacing();
    let n: usize = nx * ny * nz;

    let mut values: Vec<f64> = vec![0.0; n * max_c];
    let mut buf: [f64; 3] = [0.0, 0.0, 0.0];
    for idx in 0..n {
        arr.tuple_as_f64(idx, &mut buf);
        for c in 0..max_c {
            values[idx * max_c + c] = buf[c];
        }
    }

    let index = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };

    let mut divergence: Vec<f64> = vec![0.0; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let center = index(i, j, k);
                let mut sum = 0.0;

                for c in 0..max_c {
                    if spacing[c].abs() <= 1e-15 {
                        continue;
                    }

                    let (minus, plus) = match c {
                        0 => (
                            index(if i > 0 { i - 1 } else { i }, j, k),
                            index(if i + 1 < nx { i + 1 } else { i }, j, k),
                        ),
                        1 => (
                            index(i, if j > 0 { j - 1 } else { j }, k),
                            index(i, if j + 1 < ny { j + 1 } else { j }, k),
                        ),
                        _ => (
                            index(i, j, if k > 0 { k - 1 } else { k }),
                            index(i, j, if k + 1 < nz { k + 1 } else { k }),
                        ),
                    };

                    sum +=
                        (values[plus * max_c + c] - values[minus * max_c + c]) * 0.5 / spacing[c];
                }

                divergence[center] = sum;
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
    fn boundary_uses_replicated_pixel_half_difference() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "V",
                vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 2.0, 0.0, 0.0],
                3,
            )));

        let result = compute_divergence(&img, "V");
        let div_arr = result.point_data().get_array("Divergence").unwrap();
        let mut buf: [f64; 1] = [0.0];

        div_arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-10, "Expected 0.5, got {}", buf[0]);
        div_arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-10, "Expected 0.5, got {}", buf[0]);
    }

    #[test]
    fn two_component_field_uses_xy_components() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        let mut data: Vec<f64> = Vec::new();
        for j in 0..3 {
            for i in 0..3 {
                data.push(i as f64);
                data.push(j as f64);
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("V", data, 2)));

        let result = compute_divergence(&img, "V");
        let div_arr = result.point_data().get_array("Divergence").unwrap();
        let mut buf: [f64; 1] = [0.0];
        div_arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10, "Expected 2.0, got {}", buf[0]);
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let result = compute_divergence(&img, "NonExistent");
        assert_eq!(result.dimensions(), img.dimensions());
    }
}
