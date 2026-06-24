use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply a custom 3x3 convolution kernel to a 2D ImageData slice.
///
/// The kernel is a 3x3 array in row-major order. Applied to each XY slice.
pub fn image_convolve_3x3(input: &ImageData, scalars: &str, kernel: &[f64; 9]) -> ImageData {
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

    let get = |i: i64, j: i64, k: usize| -> f64 {
        let ii = i.clamp(0, nx as i64 - 1) as usize;
        let jj = j.clamp(0, ny as i64 - 1) as usize;
        values[k * ny * nx + jj * nx + ii]
    };

    let mut result = vec![0.0f64; n];
    let offsets: [(i64, i64); 9] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (0, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut sum = 0.0;
                for (ki, &(di, dj)) in offsets.iter().enumerate() {
                    sum += kernel[ki] * get(i as i64 + di, j as i64 + dj, k);
                }
                result[k * ny * nx + j * nx + i] = sum;
            }
        }
    }

    let mut img = input.clone();
    let mut new_attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                result.clone(),
                1,
            )));
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = new_attrs;
    img
}

/// Sharpen an ImageData using unsharp masking.
pub fn image_sharpen(input: &ImageData, scalars: &str, amount: f64) -> ImageData {
    // Sharpen kernel = identity + amount * (identity - blur)
    let a = amount;
    let kernel = [0.0, -a, 0.0, -a, 1.0 + 4.0 * a, -a, 0.0, -a, 0.0];
    image_convolve_3x3(input, scalars, &kernel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_kernel() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0],
                1,
            )));

        let identity = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        let result = image_convolve_3x3(&img, "v", &identity);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 5.0);
    }

    #[test]
    fn box_blur() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        let mut values = vec![0.0; 9];
        values[4] = 9.0; // center = 9
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let blur = [1.0 / 9.0; 9];
        let result = image_convolve_3x3(&img, "v", &blur);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10); // 9/9 = 1
    }

    #[test]
    fn sharpen_preserves_uniform() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![5.0; 25],
                1,
            )));

        let result = image_sharpen(&img, "v", 1.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = image_convolve_3x3(&img, "nope", &[0.0; 9]);
        assert_eq!(result.dimensions(), [3, 3, 1]);
    }
}
