//! Custom stencil operations on ImageData: generic convolution, median, rank.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply a custom 3D stencil/kernel to an ImageData scalar field.
///
/// `kernel` is a flat array of size (2r+1)^3 where r is the radius.
pub fn apply_stencil_3d(
    image: &ImageData,
    array_name: &str,
    kernel: &[f64],
    radius: usize,
) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];
    let kside = 2 * radius + 1;

    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut output = vec![0.0f64; n];

    for iz in 0..dims[2] {
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                let mut sum = 0.0;
                let mut ki = 0;
                for dz in 0..kside {
                    for dy in 0..kside {
                        for dx in 0..kside {
                            let nx = ix as i64 + dx as i64 - radius as i64;
                            let ny = iy as i64 + dy as i64 - radius as i64;
                            let nz = iz as i64 + dz as i64 - radius as i64;
                            if nx >= 0
                                && ny >= 0
                                && nz >= 0
                                && (nx as usize) < dims[0]
                                && (ny as usize) < dims[1]
                                && (nz as usize) < dims[2]
                            {
                                let ni = nx as usize
                                    + ny as usize * dims[0]
                                    + nz as usize * dims[0] * dims[1];
                                if ki < kernel.len() {
                                    sum += kernel[ki] * vals[ni];
                                }
                                ki += 1;
                            }
                        }
                    }
                }
                output[idx] = sum;
            }
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, 1,
        )));
    result
}

/// 3D median filter with given radius.
pub fn median_filter_3d(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];
    let r = radius as i64;

    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut output = vec![0.0f64; n];

    for iz in 0..dims[2] {
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                let mut neighbors = Vec::new();
                for dz in -r..=r {
                    for dy in -r..=r {
                        for dx in -r..=r {
                            let nx = ix as i64 + dx;
                            let ny = iy as i64 + dy;
                            let nz = iz as i64 + dz;
                            if nx >= 0
                                && ny >= 0
                                && nz >= 0
                                && (nx as usize) < dims[0]
                                && (ny as usize) < dims[1]
                                && (nz as usize) < dims[2]
                            {
                                neighbors.push(
                                    vals[nx as usize
                                        + ny as usize * dims[0]
                                        + nz as usize * dims[0] * dims[1]],
                                );
                            }
                        }
                    }
                }
                neighbors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let mid = neighbors.len() / 2;
                output[idx] = if neighbors.len() % 2 == 0 {
                    let low_mid = neighbors[mid - 1];
                    low_mid + (neighbors[mid] - low_mid) * 0.5
                } else {
                    neighbors[mid]
                };
            }
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, 1,
        )));
    result
}

/// Create common kernels.
pub fn gaussian_kernel_3d(radius: usize, sigma: f64) -> Vec<f64> {
    let side = 2 * radius + 1;
    let mut kernel = vec![0.0; side * side * side];
    let mut sum = 0.0;
    let r = radius as i64;
    for dz in -r..=r {
        for dy in -r..=r {
            for dx in -r..=r {
                let d2 = (dx * dx + dy * dy + dz * dz) as f64;
                let w = (-d2 / (2.0 * sigma * sigma)).exp();
                let idx =
                    (dx + r) as usize + (dy + r) as usize * side + (dz + r) as usize * side * side;
                kernel[idx] = w;
                sum += w;
            }
        }
    }
    for v in &mut kernel {
        *v /= sum;
    }
    kernel
}

pub fn laplacian_kernel_3d() -> Vec<f64> {
    let mut k = vec![0.0; 27];
    k[13] = -6.0; // center
    k[4] = 1.0;
    k[10] = 1.0;
    k[12] = 1.0;
    k[14] = 1.0;
    k[16] = 1.0;
    k[22] = 1.0; // 6 neighbors
    k
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gaussian_blur() {
        let img = ImageData::from_function(
            [10, 10, 10],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, y, z| {
                if x > 4.0 && x < 6.0 && y > 4.0 && y < 6.0 && z > 4.0 && z < 6.0 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let kernel = gaussian_kernel_3d(1, 1.0);
        let result = apply_stencil_3d(&img, "val", &kernel, 1);
        assert!(result.point_data().get_array("val").is_some());
    }

    #[test]
    fn convolution_boundary_advances_kernel_like_vtk() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![10.0, 20.0],
                1,
            )));

        let kernel: Vec<f64> = (0..27).map(|i| i as f64).collect();
        let result = apply_stencil_3d(&img, "val", &kernel, 1);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 20.0);
    }

    #[test]
    fn median_3d() {
        let img = ImageData::from_function(
            [8, 8, 8],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, _, _| x,
        );
        let result = median_filter_3d(&img, "val", 1);
        assert!(result.point_data().get_array("val").is_some());
    }

    #[test]
    fn median_boundary_even_count_averages_middle_values() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 10.0, 20.0, 30.0],
                1,
            )));

        let result = median_filter_3d(&img, "val", 1);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 15.0);
    }

    #[test]
    fn laplacian() {
        let k = laplacian_kernel_3d();
        assert_eq!(k.len(), 27);
        assert!((k.iter().sum::<f64>()).abs() < 1e-10); // should sum to 0
    }
}
