use crate::data::{AnyDataArray, DataArray, DataSet, ImageData};

/// Evaluate a scalar function on an ImageData grid.
///
/// The function `f(x, y, z) -> f64` is evaluated at every point of the grid,
/// producing a scalar array.
pub fn sample_function<F>(image: &ImageData, name: &str, f: F) -> ImageData
where
    F: Fn(f64, f64, f64) -> f64,
{
    let n = image.num_points();
    let mut values = Vec::with_capacity(n);

    for i in 0..n {
        let p = image.point(i);
        values.push(f(p[0], p[1], p[2]));
    }

    let mut result = image.clone();
    let arr = DataArray::from_vec(name, values, 1);
    result.point_data_mut().add_array(AnyDataArray::F64(arr));
    result.point_data_mut().set_active_scalars(name);
    result
}

/// Create an ImageData grid and evaluate a scalar function on it.
pub fn sample_function_on_bounds<F>(
    bounds: [f64; 6], // [x_min, x_max, y_min, y_max, z_min, z_max]
    dimensions: [usize; 3],
    name: &str,
    f: F,
) -> ImageData
where
    F: Fn(f64, f64, f64) -> f64,
{
    let mut image = ImageData::with_dimensions(dimensions[0], dimensions[1], dimensions[2]);
    image.set_origin([bounds[0], bounds[2], bounds[4]]);
    image.set_spacing([
        (bounds[1] - bounds[0]) / (dimensions[0] - 1).max(1) as f64,
        (bounds[3] - bounds[2]) / (dimensions[1] - 1).max(1) as f64,
        (bounds[5] - bounds[4]) / (dimensions[2] - 1).max(1) as f64,
    ]);
    sample_function(&image, name, f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_sphere_field() {
        let image = sample_function_on_bounds(
            [-1.0, 1.0, -1.0, 1.0, -1.0, 1.0],
            [5, 5, 5],
            "sphere",
            |x, y, z| x * x + y * y + z * z,
        );
        assert_eq!(image.dimensions(), [5, 5, 5]);
        let s = image.point_data().scalars().unwrap();
        assert_eq!(s.num_tuples(), 125);

        // Origin point should have value 1+1+1 = 3 (corner)
        let mut val = [0.0f64];
        s.tuple_as_f64(0, &mut val);
        assert!((val[0] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn sample_on_existing_grid() {
        let mut image = ImageData::with_dimensions(3, 3, 3);
        image.set_spacing([0.5, 0.5, 0.5]);
        let result = sample_function(&image, "linear", |x, _y, _z| x);
        let s = result.point_data().scalars().unwrap();
        assert_eq!(s.num_tuples(), 27);
    }
}
