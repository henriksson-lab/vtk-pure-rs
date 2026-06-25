use crate::data::{AnyDataArray, DataArray, ImageData};

/// Max pooling on ImageData: take the maximum in each non-overlapping block.
///
/// Reduces resolution by `pool_size` in each dimension.
pub fn image_max_pool(input: &ImageData, scalars: &str, pool_size: usize) -> ImageData {
    pool_op(input, scalars, pool_size, f64::max)
}

/// Min pooling: take the minimum in each block.
pub fn image_min_pool(input: &ImageData, scalars: &str, pool_size: usize) -> ImageData {
    pool_op(input, scalars, pool_size, f64::min)
}

/// Average pooling: take the mean of each block.
pub fn image_avg_pool(input: &ImageData, scalars: &str, pool_size: usize) -> ImageData {
    avg_pool_op(input, scalars, pool_size)
}

fn pool_op<F: Fn(f64, f64) -> f64>(
    input: &ImageData,
    scalars: &str,
    pool_size: usize,
    op: F,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let num_components = arr.num_components();
    let p = pool_size.max(1);
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let pz = if nz <= 1 { 1 } else { p };
    let spacing = input.spacing();
    let origin = input.origin();

    let nnx = vtk_shrink_dimension(nx, p);
    let nny = vtk_shrink_dimension(ny, p);
    let nnz = vtk_shrink_dimension(nz, pz);
    let mut buf = vec![0.0f64; num_components];
    let mut values = Vec::with_capacity(nnx * nny * nnz * num_components);

    for dk in 0..nnz {
        for dj in 0..nny {
            for di in 0..nnx {
                let mut result = vec![f64::NAN; num_components];
                for k in dk * pz..(dk * pz + pz).min(nz) {
                    for j in dj * p..(dj * p + p).min(ny) {
                        for i in di * p..(di * p + p).min(nx) {
                            arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
                            for c in 0..num_components {
                                result[c] = if result[c].is_nan() {
                                    buf[c]
                                } else {
                                    op(result[c], buf[c])
                                };
                            }
                        }
                    }
                }
                values.extend(result.into_iter().map(|x| if x.is_nan() { 0.0 } else { x }));
            }
        }
    }

    let new_sp = [
        spacing[0] * p as f64,
        spacing[1] * p as f64,
        spacing[2] * p as f64,
    ];
    let mut img = ImageData::with_dimensions(nnx, nny, nnz);
    img.set_origin(origin);
    img.set_spacing(new_sp);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars,
            values,
            num_components,
        )));
    img
}

fn avg_pool_op(input: &ImageData, scalars: &str, pool_size: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let num_components = arr.num_components();
    let p = pool_size.max(1);
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let pz = if nz <= 1 { 1 } else { p };
    let spacing = input.spacing();
    let origin = input.origin();

    let nnx = vtk_shrink_dimension(nx, p);
    let nny = vtk_shrink_dimension(ny, p);
    let nnz = vtk_shrink_dimension(nz, pz);
    let mut buf = vec![0.0f64; num_components];
    let mut values = Vec::with_capacity(nnx * nny * nnz * num_components);
    let norm = 1.0 / (p * p * pz) as f64;

    for dk in 0..nnz {
        for dj in 0..nny {
            for di in 0..nnx {
                let mut sum = vec![0.0; num_components];
                for k in dk * pz..(dk * pz + pz).min(nz) {
                    for j in dj * p..(dj * p + p).min(ny) {
                        for i in di * p..(di * p + p).min(nx) {
                            arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
                            for c in 0..num_components {
                                sum[c] += buf[c];
                            }
                        }
                    }
                }
                values.extend(sum.into_iter().map(|x| x * norm));
            }
        }
    }

    let new_sp = [
        spacing[0] * p as f64,
        spacing[1] * p as f64,
        spacing[2] * p as f64,
    ];
    let mut img = ImageData::with_dimensions(nnx, nny, nnz);
    img.set_origin(origin);
    img.set_spacing(new_sp);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars,
            values,
            num_components,
        )));
    img
}

fn vtk_shrink_dimension(dim: usize, factor: usize) -> usize {
    if dim <= factor {
        1
    } else {
        (dim - factor) / factor + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_pool_2x2() {
        let mut img = ImageData::with_dimensions(4, 4, 1);
        let values: Vec<f64> = (0..16).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_max_pool(&img, "v", 2);
        assert_eq!(result.dimensions(), [2, 2, 1]);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 5.0); // max of [0,1,4,5]
    }

    #[test]
    fn min_pool() {
        let mut img = ImageData::with_dimensions(4, 4, 1);
        let values: Vec<f64> = (0..16).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_min_pool(&img, "v", 2);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // min of [0,1,4,5]
    }

    #[test]
    fn pool_size_1_noop() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 9], 1)));

        let result = image_max_pool(&img, "v", 1);
        assert_eq!(result.dimensions(), [3, 3, 1]);
    }

    #[test]
    fn drops_incomplete_blocks_like_vtk_image_shrink3d() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let values: Vec<f64> = (0..25).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_max_pool(&img, "v", 2);
        assert_eq!(result.dimensions(), [2, 2, 1]);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(4, 4, 1);
        let r = image_max_pool(&img, "nope", 2);
        assert_eq!(r.dimensions(), [4, 4, 1]);
    }
}
