use crate::data::{AnyDataArray, DataArray, DataSet, ImageData};
use crate::types::ImplicitFunction;

/// Options mirroring the core vtkSampleFunction execution knobs.
#[derive(Debug, Clone)]
pub struct SampleFunctionOptions {
    pub scalar_array_name: String,
    pub normal_array_name: String,
    pub compute_normals: bool,
    pub capping: bool,
    pub cap_value: f64,
}

impl Default for SampleFunctionOptions {
    fn default() -> Self {
        Self {
            scalar_array_name: "scalars".to_string(),
            normal_array_name: "normals".to_string(),
            compute_normals: true,
            capping: false,
            cap_value: f64::MAX,
        }
    }
}

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
    let dims = [
        dimensions[0].max(1),
        dimensions[1].max(1),
        dimensions[2].max(1),
    ];
    let mut image = ImageData::with_dimensions(dims[0], dims[1], dims[2]);
    image.set_origin([bounds[0], bounds[2], bounds[4]]);
    image.set_spacing([
        if dims[0] <= 1 {
            1.0
        } else {
            (bounds[1] - bounds[0]) / (dims[0] - 1) as f64
        },
        if dims[1] <= 1 {
            1.0
        } else {
            (bounds[3] - bounds[2]) / (dims[1] - 1) as f64
        },
        if dims[2] <= 1 {
            1.0
        } else {
            (bounds[5] - bounds[4]) / (dims[2] - 1) as f64
        },
    ]);
    sample_function(&image, name, f)
}

/// Sample an implicit function over model bounds, matching vtkSampleFunction.
///
/// VTK computes points from `ModelBounds` and `SampleDimensions`, evaluates
/// function values, optionally stores negative normalized gradients as normals,
/// and optionally overwrites the six image boundary faces with `CapValue`.
pub fn sample_implicit_function_on_bounds(
    implicit_function: &dyn ImplicitFunction,
    model_bounds: [f64; 6],
    sample_dimensions: [usize; 3],
    options: &SampleFunctionOptions,
) -> ImageData {
    let dims = [
        sample_dimensions[0].max(1),
        sample_dimensions[1].max(1),
        sample_dimensions[2].max(1),
    ];
    let origin = [model_bounds[0], model_bounds[2], model_bounds[4]];
    let spacing = [
        if dims[0] <= 1 {
            1.0
        } else {
            (model_bounds[1] - model_bounds[0]) / (dims[0] - 1) as f64
        },
        if dims[1] <= 1 {
            1.0
        } else {
            (model_bounds[3] - model_bounds[2]) / (dims[1] - 1) as f64
        },
        if dims[2] <= 1 {
            1.0
        } else {
            (model_bounds[5] - model_bounds[4]) / (dims[2] - 1) as f64
        },
    ];

    let num_points = dims[0] * dims[1] * dims[2];
    let mut scalars = Vec::with_capacity(num_points);
    let mut normals = options
        .compute_normals
        .then(|| Vec::with_capacity(num_points * 3));

    for k in 0..dims[2] {
        let z = origin[2] + k as f64 * spacing[2];
        for j in 0..dims[1] {
            let y = origin[1] + j as f64 * spacing[1];
            for i in 0..dims[0] {
                let x = origin[0] + i as f64 * spacing[0];
                scalars.push(implicit_function.evaluate(x, y, z));

                if let Some(normals) = normals.as_mut() {
                    let gradient = implicit_function.gradient(x, y, z);
                    let normal = normalize(gradient);
                    normals.push(-normal[0] as f32);
                    normals.push(-normal[1] as f32);
                    normals.push(-normal[2] as f32);
                }
            }
        }
    }

    if options.capping {
        cap_boundaries(&mut scalars, dims, options.cap_value);
    }

    let mut image = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(spacing)
        .with_origin(origin);
    image
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            &options.scalar_array_name,
            scalars,
            1,
        )));
    image
        .point_data_mut()
        .set_active_scalars(&options.scalar_array_name);

    if let Some(normals) = normals {
        image
            .point_data_mut()
            .add_array(AnyDataArray::F32(DataArray::from_vec(
                &options.normal_array_name,
                normals,
                3,
            )));
        image
            .point_data_mut()
            .set_active_normals(&options.normal_array_name);
    }

    image
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn cap_boundaries(scalars: &mut [f64], dims: [usize; 3], cap_value: f64) {
    let idx = |i: usize, j: usize, k: usize| i + j * dims[0] + k * dims[0] * dims[1];

    for j in 0..dims[1] {
        for i in 0..dims[0] {
            scalars[idx(i, j, 0)] = cap_value;
            scalars[idx(i, j, dims[2] - 1)] = cap_value;
        }
    }

    for k in 0..dims[2] {
        for j in 0..dims[1] {
            scalars[idx(0, j, k)] = cap_value;
            scalars[idx(dims[0] - 1, j, k)] = cap_value;
        }
    }

    for k in 0..dims[2] {
        for i in 0..dims[0] {
            scalars[idx(i, 0, k)] = cap_value;
            scalars[idx(i, dims[1] - 1, k)] = cap_value;
        }
    }
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

    #[test]
    fn vtk_sample_function_caps_and_normals() {
        let sphere = crate::types::ImplicitSphere::new([0.0, 0.0, 0.0], 1.0);
        let opts = SampleFunctionOptions {
            capping: true,
            cap_value: 42.0,
            ..Default::default()
        };
        let image = sample_implicit_function_on_bounds(
            &sphere,
            [-1.0, 1.0, -1.0, 1.0, -1.0, 1.0],
            [3, 3, 3],
            &opts,
        );

        assert_eq!(image.dimensions(), [3, 3, 3]);
        assert!(image.point_data().normals().is_some());
        assert_eq!(image.scalar_at(0, 0, 0).unwrap(), 42.0);
        assert!((image.scalar_at(1, 1, 1).unwrap() + 1.0).abs() < 1e-10);
    }
}
