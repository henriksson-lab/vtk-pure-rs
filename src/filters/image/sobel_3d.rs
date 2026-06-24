use crate::data::{AnyDataArray, DataArray, ImageData};

/// 3D Sobel edge detection on ImageData.
///
/// Computes gradient magnitude using full 3x3x3 Sobel operators in X, Y, and Z.
/// Adds a "SobelMagnitude" point data array.
pub fn sobel_3d(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let n: usize = nx * ny * nz;

    // Read scalar values into a flat array
    let mut values: Vec<f64> = vec![0.0; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let get = |i: i64, j: i64, k: i64| -> f64 {
        let ii: usize = i.clamp(0, nx as i64 - 1) as usize;
        let jj: usize = j.clamp(0, ny as i64 - 1) as usize;
        let kk: usize = k.clamp(0, nz as i64 - 1) as usize;
        values[kk * ny * nx + jj * nx + ii]
    };

    // 3D Sobel kernels: separable as smoothing (1,2,1) x smoothing x derivative (-1,0,1)
    // Gx kernel: derivative in x, smoothing in y and z
    // Gy kernel: smoothing in x, derivative in y, smoothing in z
    // Gz kernel: smoothing in x, smoothing in y, derivative in z
    let smooth: [f64; 3] = [1.0, 2.0, 1.0];
    let deriv: [f64; 3] = [-1.0, 0.0, 1.0];

    let mut mag: Vec<f64> = vec![0.0; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let ii: i64 = i as i64;
                let jj: i64 = j as i64;
                let kk: i64 = k as i64;

                let mut gx: f64 = 0.0;
                let mut gy: f64 = 0.0;
                let mut gz: f64 = 0.0;

                for dz in 0i64..3 {
                    for dy in 0i64..3 {
                        for dx in 0i64..3 {
                            let val: f64 = get(ii + dx - 1, jj + dy - 1, kk + dz - 1);
                            gx += deriv[dx as usize]
                                * smooth[dy as usize]
                                * smooth[dz as usize]
                                * val;
                            gy += smooth[dx as usize]
                                * deriv[dy as usize]
                                * smooth[dz as usize]
                                * val;
                            gz += smooth[dx as usize]
                                * smooth[dy as usize]
                                * deriv[dz as usize]
                                * val;
                        }
                    }
                }

                mag[k * ny * nx + j * nx + i] = (gx * gx + gy * gy + gz * gz).sqrt();
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SobelMagnitude",
            mag,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_field_zero_gradient() {
        let mut img = ImageData::with_dimensions(4, 4, 4);
        let n: usize = 64;
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "scalars",
                vec![7.0; n],
                1,
            )));

        let result = sobel_3d(&img, "scalars");
        let arr = result.point_data().get_array("SobelMagnitude").unwrap();
        assert_eq!(arr.num_tuples(), n);
        let mut buf = [0.0f64];
        for i in 0..n {
            arr.tuple_as_f64(i, &mut buf);
            assert!(
                buf[0].abs() < 1e-10,
                "expected zero gradient, got {}",
                buf[0]
            );
        }
    }

    #[test]
    fn step_edge_in_x() {
        let mut img = ImageData::with_dimensions(5, 5, 5);
        let n: usize = 125;
        let mut values: Vec<f64> = vec![0.0; n];
        // Right half (x >= 3) = 100
        for k in 0..5usize {
            for j in 0..5usize {
                for i in 3..5usize {
                    values[k * 25 + j * 5 + i] = 100.0;
                }
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("s", values, 1)));

        let result = sobel_3d(&img, "s");
        let arr = result.point_data().get_array("SobelMagnitude").unwrap();

        // Interior point at edge (2,2,2) should have high magnitude
        let mut edge_val = [0.0f64];
        arr.tuple_as_f64(2 * 25 + 2 * 5 + 2, &mut edge_val);

        // Interior point far from edge (0,2,2) should be lower or zero
        let mut interior_val = [0.0f64];
        arr.tuple_as_f64(2 * 25 + 2 * 5 + 0, &mut interior_val);

        assert!(
            edge_val[0] > interior_val[0],
            "edge should have higher magnitude"
        );
    }

    #[test]
    fn missing_array_returns_copy() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let result = sobel_3d(&img, "nonexistent");
        assert!(result.point_data().get_array("SobelMagnitude").is_none());
    }
}
