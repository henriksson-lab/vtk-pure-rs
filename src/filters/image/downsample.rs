use crate::data::{AnyDataArray, DataArray, ImageData};

/// Downsample an ImageData by averaging blocks of voxels.
///
/// Reduces resolution by `factor` in each dimension by averaging
/// non-overlapping blocks. More robust than nearest-neighbor downsampling.
pub fn image_downsample(input: &ImageData, scalars: &str, factor: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let f = factor.max(1);
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let spacing = input.spacing();
    let origin = input.origin();

    let factor_z = if nz == 1 { 1 } else { f };
    let nnx = shrink_len(nx, f);
    let nny = shrink_len(ny, f);
    let nnz = shrink_len(nz, factor_z);
    let num_components = arr.num_components();
    if num_components == 0 || nnx == 0 || nny == 0 || nnz == 0 {
        return input.clone();
    }

    let mut values = vec![0.0f64; nnx * nny * nnz * num_components];
    let mut buf = vec![0.0f64; num_components];
    let norm = 1.0 / (f * f * factor_z) as f64;

    for dk in 0..nnz {
        for dj in 0..nny {
            for di in 0..nnx {
                let mut sum = vec![0.0; num_components];
                for k in dk * factor_z..dk * factor_z + factor_z {
                    for j in dj * f..dj * f + f {
                        for i in di * f..di * f + f {
                            arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
                            for comp in 0..num_components {
                                sum[comp] += buf[comp];
                            }
                        }
                    }
                }
                let out_idx = (dk * nny * nnx + dj * nnx + di) * num_components;
                for comp in 0..num_components {
                    values[out_idx + comp] = sum[comp] * norm;
                }
            }
        }
    }

    let new_spacing = [
        spacing[0] * f as f64,
        spacing[1] * f as f64,
        spacing[2] * f as f64,
    ];
    let mut img = ImageData::with_dimensions(nnx, nny, nnz);
    img.set_origin(origin);
    img.set_spacing(new_spacing);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars,
            values,
            num_components,
        )));
    img
}

fn shrink_len(len: usize, factor: usize) -> usize {
    if len < factor {
        0
    } else {
        len / factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downsample_2x() {
        let mut img = ImageData::with_dimensions(4, 4, 1);
        let values: Vec<f64> = (0..16).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_downsample(&img, "v", 2);
        assert_eq!(result.dimensions(), [2, 2, 1]);
    }

    #[test]
    fn averaging() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 10.0, 20.0, 30.0],
                1,
            )));

        let result = image_downsample(&img, "v", 2);
        assert_eq!(result.dimensions(), [1, 1, 1]);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 15.0).abs() < 1e-10); // (0+10+20+30)/4
    }

    #[test]
    fn factor_1_noop() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 9], 1)));

        let result = image_downsample(&img, "v", 1);
        assert_eq!(result.dimensions(), [3, 3, 1]);
    }

    #[test]
    fn excludes_trailing_partial_blocks() {
        let mut img = ImageData::with_dimensions(5, 5, 2);
        let values: Vec<f64> = (0..50).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_downsample(&img, "v", 2);
        assert_eq!(result.dimensions(), [2, 2, 1]);
    }
}
