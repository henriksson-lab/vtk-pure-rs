use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply Gaussian smoothing to an ImageData scalar field.
///
/// Performs a 3D Gaussian blur with the given `sigma` (in voxel units)
/// and `radius` (kernel half-size in voxels). Uses separable 1D passes
/// along each axis for efficiency.
pub fn image_gaussian_smooth(
    input: &ImageData,
    scalars: &str,
    sigma: f64,
    radius: usize,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let num_components = arr.num_components();

    let mut values = vec![0.0f64; n * num_components];
    let mut buf = vec![0.0f64; num_components];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let offset = i * num_components;
        values[offset..offset + num_components].copy_from_slice(&buf);
    }

    // Build 1D Gaussian kernel
    let sigma = sigma.max(0.1);
    let r = radius.max(1);
    let kernel_size = 2 * r + 1;
    let mut kernel = vec![0.0f64; kernel_size];
    let mut sum = 0.0;
    for i in 0..kernel_size {
        let x = (i as f64) - r as f64;
        let w = (-x * x / (2.0 * sigma * sigma)).exp();
        kernel[i] = w;
        sum += w;
    }
    for w in &mut kernel {
        *w /= sum;
    }

    // Separable: X pass
    let mut tmp = vec![0.0f64; n * num_components];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let dst_idx = k * ny * nx + j * nx + i;
                for c in 0..num_components {
                    let mut acc = 0.0;
                    for ki in 0..kernel_size {
                        let ii = (i as i64 + ki as i64 - r as i64).clamp(0, nx as i64 - 1) as usize;
                        acc +=
                            values[(k * ny * nx + j * nx + ii) * num_components + c] * kernel[ki];
                    }
                    tmp[dst_idx * num_components + c] = acc;
                }
            }
        }
    }

    // Y pass
    let mut tmp2 = vec![0.0f64; n * num_components];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let dst_idx = k * ny * nx + j * nx + i;
                for c in 0..num_components {
                    let mut acc = 0.0;
                    for ki in 0..kernel_size {
                        let jj = (j as i64 + ki as i64 - r as i64).clamp(0, ny as i64 - 1) as usize;
                        acc += tmp[(k * ny * nx + jj * nx + i) * num_components + c] * kernel[ki];
                    }
                    tmp2[dst_idx * num_components + c] = acc;
                }
            }
        }
    }

    // Z pass
    let mut result = vec![0.0f64; n * num_components];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let dst_idx = k * ny * nx + j * nx + i;
                for c in 0..num_components {
                    let mut acc = 0.0;
                    for ki in 0..kernel_size {
                        let kk = (k as i64 + ki as i64 - r as i64).clamp(0, nz as i64 - 1) as usize;
                        acc += tmp2[(kk * ny * nx + j * nx + i) * num_components + c] * kernel[ki];
                    }
                    result[dst_idx * num_components + c] = acc;
                }
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
                num_components,
            )));
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = new_attrs;
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_spike_image() -> ImageData {
        let mut img = ImageData::with_dimensions(5, 5, 5);
        let n = 125;
        let mut values = vec![0.0f64; n];
        // Spike at center
        values[62] = 100.0; // (2,2,2)
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", values, 1)));
        img
    }

    #[test]
    fn smoothing_reduces_spike() {
        let img = make_spike_image();
        let result = image_gaussian_smooth(&img, "val", 1.0, 1);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(62, &mut buf);
        // Spike should be reduced
        assert!(buf[0] < 100.0, "center={}", buf[0]);
        assert!(buf[0] > 0.0);
    }

    #[test]
    fn smoothing_spreads() {
        let img = make_spike_image();
        let result = image_gaussian_smooth(&img, "val", 1.0, 1);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        // Neighbor should pick up some value
        arr.tuple_as_f64(63, &mut buf); // (3,2,2)
        assert!(buf[0] > 0.0);
    }

    #[test]
    fn preserves_uniform() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![5.0; 27],
                1,
            )));
        let result = image_gaussian_smooth(&img, "val", 1.0, 1);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        for i in 0..27 {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - 5.0).abs() < 1e-10);
        }
    }

    #[test]
    fn preserves_components() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![1.0, 10.0, 3.0, 30.0, 5.0, 50.0],
                2,
            )));
        let result = image_gaussian_smooth(&img, "val", 1.0, 1);
        let arr = result.point_data().get_array("val").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(1, &mut buf);
        assert!(buf[0] > 1.0 && buf[0] < 5.0);
        assert!(buf[1] > 10.0 && buf[1] < 50.0);
    }
}
