use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply Sobel edge detection to a 2D or 3D ImageData scalar field.
///
/// Computes gradient magnitude using 3x3(x3) Sobel kernels.
/// Adds a "SobelMagnitude" scalar array.
pub fn image_sobel(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;

    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let get = |i: i64, j: i64, k: i64| -> f64 {
        let ii = i.clamp(0, nx as i64 - 1) as usize;
        let jj = j.clamp(0, ny as i64 - 1) as usize;
        let kk = k.clamp(0, nz as i64 - 1) as usize;
        values[kk * ny * nx + jj * nx + ii]
    };

    let mut mag = vec![0.0f64; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let ii = i as i64;
                let jj = j as i64;
                let kk = k as i64;

                // Sobel X: [-1 0 1; -2 0 2; -1 0 1]
                let gx = -get(ii - 1, jj - 1, kk) + get(ii + 1, jj - 1, kk)
                    - 2.0 * get(ii - 1, jj, kk)
                    + 2.0 * get(ii + 1, jj, kk)
                    - get(ii - 1, jj + 1, kk)
                    + get(ii + 1, jj + 1, kk);

                // Sobel Y
                let gy =
                    -get(ii - 1, jj - 1, kk) - 2.0 * get(ii, jj - 1, kk) - get(ii + 1, jj - 1, kk)
                        + get(ii - 1, jj + 1, kk)
                        + 2.0 * get(ii, jj + 1, kk)
                        + get(ii + 1, jj + 1, kk);

                // Sobel Z (for 3D)
                let gz = if nz > 1 {
                    -get(ii, jj, kk - 1) + get(ii, jj, kk + 1)
                } else {
                    0.0
                };

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
    fn step_edge() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![0.0f64; 25];
        // Left half = 0, right half = 100
        for j in 0..5 {
            for i in 3..5 {
                values[j * 5 + i] = 100.0;
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_sobel(&img, "v");
        let arr = result.point_data().get_array("SobelMagnitude").unwrap();
        let mut buf = [0.0f64];
        // Edge at column 2-3 should have high magnitude
        arr.tuple_as_f64(2 * 1 + 2 * 5, &mut buf); // (2,2)
        let edge_mag = buf[0];
        // Interior should have zero
        arr.tuple_as_f64(0, &mut buf);
        assert!(edge_mag > buf[0]);
    }

    #[test]
    fn uniform_zero_edges() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![5.0; 9], 1)));

        let result = image_sobel(&img, "v");
        let arr = result.point_data().get_array("SobelMagnitude").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0].abs() < 1e-10);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = image_sobel(&img, "nope");
        assert!(result.point_data().get_array("SobelMagnitude").is_none());
    }
}
