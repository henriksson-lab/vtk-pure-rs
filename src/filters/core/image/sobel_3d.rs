use crate::data::{AnyDataArray, DataArray, ImageData};

/// 3D Sobel edge detection on ImageData.
///
/// Computes the 3-component Sobel gradient used by vtkImageSobel3D.
/// Adds a "Sobel" point data vector array.
pub fn sobel_3d(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        None => return input.clone(),
        _ => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0];
    let ny: usize = dims[1];
    let nz: usize = dims[2];
    let n: usize = nx * ny * nz;
    if n == 0 {
        return input.clone();
    }

    let mut values: Vec<f64> = vec![0.0; n];
    let mut buf = [0.0f64];
    for i in 0..n.min(arr.num_tuples()) {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let get = |i: i64, j: i64, k: i64| -> f64 {
        let ii: usize = i.clamp(0, nx as i64 - 1) as usize;
        let jj: usize = j.clamp(0, ny as i64 - 1) as usize;
        let kk: usize = k.clamp(0, nz as i64 - 1) as usize;
        values[kk * ny * nx + jj * nx + ii]
    };

    let spacing = input.spacing();
    let scale = [
        if spacing[0] != 0.0 {
            0.060445 / spacing[0]
        } else {
            0.0
        },
        if spacing[1] != 0.0 {
            0.060445 / spacing[1]
        } else {
            0.0
        },
        if spacing[2] != 0.0 {
            0.060445 / spacing[2]
        } else {
            0.0
        },
    ];
    let mut gradient: Vec<f64> = vec![0.0; n * 3];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let ii: i64 = i as i64;
                let jj: i64 = j as i64;
                let kk: i64 = k as i64;

                let gx = (2.0 * (get(ii + 1, jj, kk) - get(ii - 1, jj, kk))
                    + get(ii + 1, jj - 1, kk)
                    + get(ii + 1, jj + 1, kk)
                    + get(ii + 1, jj, kk - 1)
                    + get(ii + 1, jj, kk + 1)
                    - get(ii - 1, jj - 1, kk)
                    - get(ii - 1, jj + 1, kk)
                    - get(ii - 1, jj, kk - 1)
                    - get(ii - 1, jj, kk + 1)
                    + 0.586
                        * (get(ii + 1, jj - 1, kk - 1)
                            + get(ii + 1, jj - 1, kk + 1)
                            + get(ii + 1, jj + 1, kk - 1)
                            + get(ii + 1, jj + 1, kk + 1)
                            - get(ii - 1, jj - 1, kk - 1)
                            - get(ii - 1, jj - 1, kk + 1)
                            - get(ii - 1, jj + 1, kk - 1)
                            - get(ii - 1, jj + 1, kk + 1)))
                    * scale[0];

                let gy = (2.0 * (get(ii, jj + 1, kk) - get(ii, jj - 1, kk))
                    + get(ii - 1, jj + 1, kk)
                    + get(ii + 1, jj + 1, kk)
                    + get(ii, jj + 1, kk - 1)
                    + get(ii, jj + 1, kk + 1)
                    - get(ii - 1, jj - 1, kk)
                    - get(ii + 1, jj - 1, kk)
                    - get(ii, jj - 1, kk - 1)
                    - get(ii, jj - 1, kk + 1)
                    + 0.586
                        * (get(ii - 1, jj + 1, kk - 1)
                            + get(ii - 1, jj + 1, kk + 1)
                            + get(ii + 1, jj + 1, kk - 1)
                            + get(ii + 1, jj + 1, kk + 1)
                            - get(ii - 1, jj - 1, kk - 1)
                            - get(ii - 1, jj - 1, kk + 1)
                            - get(ii + 1, jj - 1, kk - 1)
                            - get(ii + 1, jj - 1, kk + 1)))
                    * scale[1];

                let gz = (2.0 * (get(ii, jj, kk + 1) - get(ii, jj, kk - 1))
                    + get(ii - 1, jj, kk + 1)
                    + get(ii + 1, jj, kk + 1)
                    + get(ii, jj - 1, kk + 1)
                    + get(ii, jj + 1, kk + 1)
                    - get(ii - 1, jj, kk - 1)
                    - get(ii + 1, jj, kk - 1)
                    - get(ii, jj - 1, kk - 1)
                    - get(ii, jj + 1, kk - 1)
                    + 0.586
                        * (get(ii - 1, jj - 1, kk + 1)
                            + get(ii - 1, jj + 1, kk + 1)
                            + get(ii + 1, jj - 1, kk + 1)
                            + get(ii + 1, jj + 1, kk + 1)
                            - get(ii - 1, jj - 1, kk - 1)
                            - get(ii - 1, jj + 1, kk - 1)
                            - get(ii + 1, jj - 1, kk - 1)
                            - get(ii + 1, jj + 1, kk - 1)))
                    * scale[2];

                let out = (k * ny * nx + j * nx + i) * 3;
                gradient[out] = gx;
                gradient[out + 1] = gy;
                gradient[out + 2] = gz;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Sobel", gradient, 3)));
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
        let arr = result.point_data().get_array("Sobel").unwrap();
        assert_eq!(arr.num_components(), 3);
        assert_eq!(arr.num_tuples(), n);
        let mut buf = [0.0f64; 3];
        for i in 0..n {
            arr.tuple_as_f64(i, &mut buf);
            assert!(
                buf[0].abs() < 1e-10,
                "expected zero x-gradient, got {}",
                buf[0]
            );
            assert!(buf[1].abs() < 1e-10);
            assert!(buf[2].abs() < 1e-10);
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
        let arr = result.point_data().get_array("Sobel").unwrap();

        // Interior point at edge (2,2,2) should have high magnitude
        let mut edge_val = [0.0f64; 3];
        arr.tuple_as_f64(2 * 25 + 2 * 5 + 2, &mut edge_val);
        let edge_mag =
            (edge_val[0] * edge_val[0] + edge_val[1] * edge_val[1] + edge_val[2] * edge_val[2])
                .sqrt();

        // Interior point far from edge (0,2,2) should be lower or zero
        let mut interior_val = [0.0f64; 3];
        arr.tuple_as_f64(2 * 25 + 2 * 5 + 0, &mut interior_val);
        let interior_mag = (interior_val[0] * interior_val[0]
            + interior_val[1] * interior_val[1]
            + interior_val[2] * interior_val[2])
            .sqrt();

        assert!(edge_mag > interior_mag, "edge should have higher magnitude");
    }

    #[test]
    fn missing_array_returns_copy() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let result = sobel_3d(&img, "nonexistent");
        assert!(result.point_data().get_array("Sobel").is_none());
    }
}
