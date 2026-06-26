use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply a separable 1D convolution kernel along each axis.
///
/// The kernel is applied sequentially: first X, then Y, then Z.
/// More efficient than a full 3D convolution for separable kernels.
pub fn image_convolve_separable(input: &ImageData, scalars: &str, kernel: &[f64]) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let r = kernel.len() / 2;

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    // X pass
    let mut tmp = vec![0.0f64; n];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut sum = 0.0;
                for (ki, &w) in kernel.iter().enumerate() {
                    let ii = (i as i64 + ki as i64 - r as i64).clamp(0, nx as i64 - 1) as usize;
                    sum += values[k * ny * nx + j * nx + ii] * w;
                }
                tmp[k * ny * nx + j * nx + i] = sum;
            }
        }
    }

    // Y pass
    let mut tmp2 = vec![0.0f64; n];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut sum = 0.0;
                for (ki, &w) in kernel.iter().enumerate() {
                    let jj = (j as i64 + ki as i64 - r as i64).clamp(0, ny as i64 - 1) as usize;
                    sum += tmp[k * ny * nx + jj * nx + i] * w;
                }
                tmp2[k * ny * nx + j * nx + i] = sum;
            }
        }
    }

    // Z pass
    let mut result = vec![0.0f64; n];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut sum = 0.0;
                for (ki, &w) in kernel.iter().enumerate() {
                    let kk = (k as i64 + ki as i64 - r as i64).clamp(0, nz as i64 - 1) as usize;
                    sum += tmp2[kk * ny * nx + j * nx + i] * w;
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

/// Create a 1D box kernel of given radius (uniform averaging).
pub fn box_kernel(radius: usize) -> Vec<f64> {
    let size = 2 * radius + 1;
    vec![1.0 / size as f64; size]
}

/// Create a 1D Gaussian kernel with given sigma and radius.
pub fn gaussian_kernel(sigma: f64, radius: usize) -> Vec<f64> {
    let size = 2 * radius + 1;
    let mut kernel = Vec::with_capacity(size);
    let mut sum = 0.0;
    for i in 0..size {
        let x = i as f64 - radius as f64;
        let w = (-x * x / (2.0 * sigma * sigma)).exp();
        kernel.push(w);
        sum += w;
    }
    for w in &mut kernel {
        *w /= sum;
    }
    kernel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_blur() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        let mut values = vec![0.0; 5];
        values[2] = 9.0;
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let k = box_kernel(1);
        let result = image_convolve_separable(&img, "v", &k);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert!(buf[0] < 9.0); // blurred
        assert!(buf[0] > 0.0);
    }

    #[test]
    fn identity_kernel() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0],
                1,
            )));
        let k = vec![0.0, 1.0, 0.0]; // identity
        let result = image_convolve_separable(&img, "v", &k);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn gaussian_kernel_sums_to_1() {
        let k = gaussian_kernel(1.0, 3);
        let sum: f64 = k.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = image_convolve_separable(&img, "nope", &[1.0]);
        assert_eq!(r.dimensions(), [3, 1, 1]);
    }
}
