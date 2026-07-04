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
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let p = pool_size.max(1);
    let pz = if nz <= 1 { 1 } else { p };
    let spacing = input.spacing();
    let origin = input.origin();

    let nnx = vtk_shrink_dimension(nx, p);
    let nny = vtk_shrink_dimension(ny, p);
    let nnz = vtk_shrink_dimension(nz, pz);
    let num_components = arr.num_components();
    if num_components == 0 || arr.num_tuples() < nx * ny * nz {
        return input.clone();
    }
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
                            for comp in 0..num_components {
                                result[comp] = if result[comp].is_nan() {
                                    buf[comp]
                                } else {
                                    op(result[comp], buf[comp])
                                };
                            }
                        }
                    }
                }
                values.extend(
                    result
                        .iter()
                        .map(|&value| if value.is_nan() { 0.0 } else { value }),
                );
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
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let p = pool_size.max(1);
    let pz = if nz <= 1 { 1 } else { p };
    let spacing = input.spacing();
    let origin = input.origin();

    let nnx = vtk_shrink_dimension(nx, p);
    let nny = vtk_shrink_dimension(ny, p);
    let nnz = vtk_shrink_dimension(nz, pz);
    let num_components = arr.num_components();
    if num_components == 0 || arr.num_tuples() < nx * ny * nz {
        return input.clone();
    }
    let mut buf = vec![0.0f64; num_components];
    let mut values = Vec::with_capacity(nnx * nny * nnz * num_components);

    for dk in 0..nnz {
        for dj in 0..nny {
            for di in 0..nnx {
                let mut sum = vec![0.0; num_components];
                for k in dk * pz..(dk * pz + pz).min(nz) {
                    for j in dj * p..(dj * p + p).min(ny) {
                        for i in di * p..(di * p + p).min(nx) {
                            arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
                            for comp in 0..num_components {
                                sum[comp] += buf[comp];
                            }
                        }
                    }
                }
                let norm = 1.0 / (p * p * pz) as f64;
                values.extend(sum.iter().map(|value| value * norm));
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
    fn missing_array() {
        let img = ImageData::with_dimensions(4, 4, 1);
        let r = image_max_pool(&img, "nope", 2);
        assert_eq!(r.dimensions(), [4, 4, 1]);
    }

    #[test]
    fn max_pool_processes_all_components() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 10.0, 2.0, 9.0, 3.0, 8.0, 4.0, 7.0],
                2,
            )));

        let result = image_max_pool(&img, "v", 2);
        let arr = result.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0f64, 0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [4.0, 10.0]);
    }

    #[test]
    fn min_pool_processes_all_components() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 10.0, 2.0, 9.0, 3.0, 8.0, 4.0, 7.0],
                2,
            )));

        let result = image_min_pool(&img, "v", 2);
        let arr = result.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0f64, 0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [1.0, 7.0]);
    }
}
