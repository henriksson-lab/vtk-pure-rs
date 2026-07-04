use crate::data::{AnyDataArray, DataArray, DataSet, ImageData};
use crate::types::ImplicitFunction;

/// Options mirroring vtkSampleImplicitFunctionFilter.
#[derive(Debug, Clone)]
pub struct SampleImplicitFunctionFilterOptions {
    pub scalar_array_name: String,
    pub gradient_array_name: String,
    pub compute_gradients: bool,
}

impl Default for SampleImplicitFunctionFilterOptions {
    fn default() -> Self {
        Self {
            scalar_array_name: "Implicit scalars".to_string(),
            gradient_array_name: "Implicit gradients".to_string(),
            compute_gradients: true,
        }
    }
}

/// Sample an implicit function over an existing dataset.
///
/// This follows vtkSampleImplicitFunctionFilter: the output keeps the input
/// geometry, topology, point data, and cell data, then appends active scalar
/// function values and optional active gradient vectors at each input point.
pub fn sample_implicit_function_filter<T>(
    input: &T,
    implicit_function: &dyn ImplicitFunction,
    options: &SampleImplicitFunctionFilterOptions,
) -> T
where
    T: DataSet + Clone,
{
    let num_points = input.num_points();
    let mut output = input.clone();
    if num_points < 1 {
        return output;
    }

    let mut scalars = Vec::with_capacity(num_points);
    let mut gradients = options
        .compute_gradients
        .then(|| Vec::with_capacity(num_points * 3));

    for point_id in 0..num_points {
        let x = input.point(point_id);
        scalars.push(implicit_function.evaluate(x[0], x[1], x[2]) as f32);

        if let Some(gradients) = gradients.as_mut() {
            let g = implicit_function.gradient(x[0], x[1], x[2]);
            gradients.push(g[0] as f32);
            gradients.push(g[1] as f32);
            gradients.push(g[2] as f32);
        }
    }

    output
        .point_data_mut()
        .add_array(AnyDataArray::F32(DataArray::from_vec(
            &options.scalar_array_name,
            scalars,
            1,
        )));
    output
        .point_data_mut()
        .set_active_scalars(&options.scalar_array_name);

    if let Some(gradients) = gradients {
        output
            .point_data_mut()
            .add_array(AnyDataArray::F32(DataArray::from_vec(
                &options.gradient_array_name,
                gradients,
                3,
            )));
        output
            .point_data_mut()
            .set_active_vectors(&options.gradient_array_name);
    }

    output
}

/// Sample an implicit function on a new ImageData grid.
///
/// Evaluates `func(x, y, z)` at each grid point and stores the result
/// as a scalar point data array.
///
/// This is a convenience helper used by grid/isosurface code. For the VTK
/// filter equivalent, use [`sample_implicit_function_filter`].
pub fn sample_implicit_function(
    dims: [usize; 3],
    spacing: [f64; 3],
    origin: [f64; 3],
    name: &str,
    func: &dyn ImplicitFunction,
) -> ImageData {
    let mut values = Vec::with_capacity(dims[0] * dims[1] * dims[2]);
    for k in 0..dims[2] {
        for j in 0..dims[1] {
            for i in 0..dims[0] {
                let x = origin[0] + i as f64 * spacing[0];
                let y = origin[1] + j as f64 * spacing[1];
                let z = origin[2] + k as f64 * spacing[2];
                values.push(func.evaluate(x, y, z));
            }
        }
    }

    let mut img = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(spacing)
        .with_origin(origin);
    let arr = DataArray::from_vec(name, values, 1);
    img.point_data_mut().add_array(AnyDataArray::F64(arr));
    img.point_data_mut().set_active_scalars(name);
    img
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;
    use crate::types::{ImplicitPlane, ImplicitSphere};

    #[test]
    fn sphere_distance_field() {
        let sphere = ImplicitSphere::new([0.0, 0.0, 0.0], 1.0);
        let img = sample_implicit_function(
            [5, 5, 5],
            [0.5, 0.5, 0.5],
            [-1.0, -1.0, -1.0],
            "distance",
            &sphere,
        );
        assert_eq!(img.dimensions(), [5, 5, 5]);
        let scalars = img.point_data().scalars().unwrap();
        assert_eq!(scalars.num_tuples(), 125);
        // Center voxel (2,2,2) at origin should have negative value (inside sphere)
        let center_val = img.scalar_at(2, 2, 2).unwrap();
        assert!(
            center_val < 0.0,
            "center should be inside sphere, got {center_val}"
        );
    }

    #[test]
    fn plane_field() {
        let plane = ImplicitPlane::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let img = sample_implicit_function(
            [5, 1, 1],
            [1.0, 1.0, 1.0],
            [-2.0, 0.0, 0.0],
            "plane",
            &plane,
        );
        // At x=-2: negative, at x=2: positive
        let v0 = img.scalar_at(0, 0, 0).unwrap();
        let v4 = img.scalar_at(4, 0, 0).unwrap();
        assert!(v0 < 0.0);
        assert!(v4 > 0.0);
    }

    #[test]
    fn vtk_filter_preserves_dataset_and_adds_arrays() {
        let input = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let sphere = ImplicitSphere::new([0.0, 0.0, 0.0], 1.0);
        let output = sample_implicit_function_filter(&input, &sphere, &Default::default());

        assert_eq!(output.points.len(), input.points.len());
        assert_eq!(output.polys.num_cells(), input.polys.num_cells());
        assert!(output.point_data().scalars().is_some());
        assert!(output.point_data().vectors().is_some());
    }

    #[test]
    fn vtk_filter_can_skip_gradients() {
        let input = PolyData::from_vertices(vec![[0.0, 0.0, 0.0]]);
        let plane = ImplicitPlane::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let opts = SampleImplicitFunctionFilterOptions {
            compute_gradients: false,
            ..Default::default()
        };
        let output = sample_implicit_function_filter(&input, &plane, &opts);

        assert!(output.point_data().scalars().is_some());
        assert!(output.point_data().vectors().is_none());
    }
}
