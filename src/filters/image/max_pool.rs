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
    // Average pooling is handled by image_downsample, but provide here for API consistency
    crate::filters::image::downsample::image_downsample(input, scalars, pool_size)
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
    let p = pool_size.max(1);
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let spacing = input.spacing();
    let origin = input.origin();

    let nnx = (nx + p - 1) / p;
    let nny = (ny + p - 1) / p;
    let nnz = (nz + p - 1) / p;
    let mut buf = [0.0f64];
    let mut values = Vec::with_capacity(nnx * nny * nnz);

    for dk in 0..nnz {
        for dj in 0..nny {
            for di in 0..nnx {
                let mut result = f64::NAN;
                for k in dk * p..(dk * p + p).min(nz) {
                    for j in dj * p..(dj * p + p).min(ny) {
                        for i in di * p..(di * p + p).min(nx) {
                            arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
                            result = if result.is_nan() {
                                buf[0]
                            } else {
                                op(result, buf[0])
                            };
                        }
                    }
                }
                values.push(if result.is_nan() { 0.0 } else { result });
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
        .add_array(AnyDataArray::F64(DataArray::from_vec(scalars, values, 1)));
    img
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
}
