use crate::data::{AnyDataArray, DataArray, ImageData, KdTree, PolyData};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityEstimate {
    FixedRadius,
    RelativeRadius,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityForm {
    VolumeNormalized,
    NumberOfPoints,
}

#[derive(Debug, Clone)]
pub struct PointDensityOptions {
    pub sample_dimensions: [usize; 3],
    pub model_bounds: [f64; 6],
    pub adjust_distance: f64,
    pub density_estimate: DensityEstimate,
    pub density_form: DensityForm,
    pub radius: f64,
    pub relative_radius: f64,
    pub scalar_weighting: bool,
    pub compute_gradient: bool,
}

impl Default for PointDensityOptions {
    fn default() -> Self {
        Self {
            sample_dimensions: [100, 100, 100],
            model_bounds: [0.0; 6],
            adjust_distance: 0.10,
            density_estimate: DensityEstimate::RelativeRadius,
            density_form: DensityForm::NumberOfPoints,
            radius: 1.0,
            relative_radius: 1.0,
            scalar_weighting: false,
            compute_gradient: false,
        }
    }
}

/// Produce a density volume from an input point cloud, following VTK's
/// vtkPointDensityFilter.
pub fn point_density_filter(input: &PolyData, options: &PointDensityOptions) -> ImageData {
    let npts = input.points.len();
    if npts == 0 || options.sample_dimensions.iter().any(|&d| d <= 1) {
        return ImageData::new();
    }

    let dims = options.sample_dimensions;
    let (origin, spacing) = compute_model_bounds(input, options, dims);
    let radius = match options.density_estimate {
        DensityEstimate::FixedRadius => options.radius.max(0.0),
        DensityEstimate::RelativeRadius => {
            let diag =
                (spacing[0] * spacing[0] + spacing[1] * spacing[1] + spacing[2] * spacing[2])
                    .sqrt();
            options.relative_radius.max(0.0) * diag
        }
    };
    let volume = (4.0 / 3.0) * std::f64::consts::PI * radius * radius * radius;
    let pts = input.points.to_vec();
    let tree = KdTree::build(&pts);
    let weights = if options.scalar_weighting {
        active_scalar_weights(input, npts)
    } else {
        None
    };

    let mut density = vec![0.0f32; dims[0] * dims[1] * dims[2]];
    for k in 0..dims[2] {
        let x2 = origin[2] + k as f64 * spacing[2];
        for j in 0..dims[1] {
            let x1 = origin[1] + j as f64 * spacing[1];
            for i in 0..dims[0] {
                let x = [origin[0] + i as f64 * spacing[0], x1, x2];
                let pids = tree.find_within_radius(x, radius);
                let d = match weights.as_deref() {
                    Some(weights) => pids.iter().map(|&(pid, _)| weights[pid]).sum(),
                    None => pids.len() as f64,
                };
                let d = match options.density_form {
                    DensityForm::NumberOfPoints => d,
                    DensityForm::VolumeNormalized => {
                        if volume > 0.0 {
                            d / volume
                        } else {
                            0.0
                        }
                    }
                };
                density[i + j * dims[0] + k * dims[0] * dims[1]] = d as f32;
            }
        }
    }

    let mut output = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_origin(origin)
        .with_spacing(spacing)
        .with_point_array(AnyDataArray::F32(DataArray::from_vec(
            "Density", density, 1,
        )));

    if options.compute_gradient {
        add_density_gradients(&mut output);
    }

    output
}

/// Compute local point density for each point in a point cloud.
///
/// For each point, counts the number of neighbors within `radius`
/// and stores it as a "Density" scalar. Also computes a normalized
/// density by dividing by the sphere volume.
pub fn point_cloud_density(input: &PolyData, radius: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let pts = input.points.to_vec();
    let tree = KdTree::build(&pts);

    let sphere_vol = (4.0 / 3.0) * std::f64::consts::PI * radius * radius * radius;
    let inv_vol = if sphere_vol > 1e-15 {
        1.0 / sphere_vol
    } else {
        0.0
    };
    let mut counts = Vec::with_capacity(n);
    let mut densities = Vec::with_capacity(n);

    for i in 0..n {
        let count = tree.find_within_radius(pts[i], radius).len();
        counts.push(count as f64);
        densities.push(count as f64 * inv_vol);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NeighborCount",
            counts,
            1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Density", densities, 1,
        )));
    pd
}

fn active_scalar_weights(input: &PolyData, npts: usize) -> Option<Vec<f64>> {
    let scalars = input.point_data().scalars()?;
    if scalars.num_tuples() != npts {
        return None;
    }

    let mut weights = Vec::with_capacity(npts);
    let mut buf = [0.0f64];
    for i in 0..npts {
        scalars.tuple_as_f64(i, &mut buf);
        weights.push(buf[0]);
    }
    Some(weights)
}

fn compute_model_bounds(
    input: &PolyData,
    options: &PointDensityOptions,
    dims: [usize; 3],
) -> ([f64; 3], [f64; 3]) {
    let mut model_bounds = options.model_bounds;
    if model_bounds[0] >= model_bounds[1]
        || model_bounds[2] >= model_bounds[3]
        || model_bounds[4] >= model_bounds[5]
    {
        let bounds = input.points.bounds();
        let raw_bounds = [
            bounds.x_min,
            bounds.x_max,
            bounds.y_min,
            bounds.y_max,
            bounds.z_min,
            bounds.z_max,
        ];
        for i in 0..3 {
            let length =
                (1.0 + options.adjust_distance) * (raw_bounds[2 * i + 1] - raw_bounds[2 * i]) / 2.0;
            let center = (raw_bounds[2 * i + 1] + raw_bounds[2 * i]) / 2.0;
            model_bounds[2 * i] = center - length;
            model_bounds[2 * i + 1] = center + length;
        }
    }

    let origin = [model_bounds[0], model_bounds[2], model_bounds[4]];
    let mut spacing = [1.0; 3];
    for i in 0..3 {
        if dims[i] > 1 {
            spacing[i] = (model_bounds[2 * i + 1] - model_bounds[2 * i]) / (dims[i] - 1) as f64;
        }
        if spacing[i] <= 0.0 {
            spacing[i] = 1.0;
        }
    }
    (origin, spacing)
}

fn add_density_gradients(output: &mut ImageData) {
    let dims = output.dimensions();
    let spacing = output.spacing();
    let Some(density_array) = output.point_data().get_array("Density") else {
        return;
    };
    let mut density = vec![0.0f64; density_array.num_tuples()];
    let mut buf = [0.0f64];
    for (idx, d) in density.iter_mut().enumerate() {
        density_array.tuple_as_f64(idx, &mut buf);
        *d = buf[0];
    }

    let mut gradients = vec![0.0f32; 3 * density.len()];
    let mut gradient_mag = vec![0.0f32; density.len()];
    let mut classification = vec![0u8; density.len()];
    let incs = [1, dims[0], dims[0] * dims[1]];

    for k in 0..dims[2] {
        for j in 0..dims[1] {
            for i in 0..dims[0] {
                let idx = i + j * dims[0] + k * dims[0] * dims[1];
                let ijk = [i, j, k];
                let mut non_zero_comp = false;
                let mut grad = [0.0f64; 3];
                for axis in 0..3 {
                    let (dm, dp, factor) = if ijk[axis] == 0 {
                        (density[idx], density[idx + incs[axis]], 1.0)
                    } else if ijk[axis] == dims[axis] - 1 {
                        (density[idx - incs[axis]], density[idx], 1.0)
                    } else {
                        (density[idx - incs[axis]], density[idx + incs[axis]], 0.5)
                    };
                    grad[axis] = factor * (dp - dm) / spacing[axis];
                    non_zero_comp = if dp != 0.0 || dm != 0.0 {
                        true
                    } else {
                        non_zero_comp
                    };
                }

                gradients[3 * idx] = grad[0] as f32;
                gradients[3 * idx + 1] = grad[1] as f32;
                gradients[3 * idx + 2] = grad[2] as f32;
                if non_zero_comp {
                    gradient_mag[idx] =
                        (grad[0] * grad[0] + grad[1] * grad[1] + grad[2] * grad[2]).sqrt() as f32;
                    classification[idx] = 1;
                }
            }
        }
    }

    output
        .point_data_mut()
        .add_array(AnyDataArray::F32(DataArray::from_vec(
            "Gradient", gradients, 3,
        )));
    output
        .point_data_mut()
        .add_array(AnyDataArray::F32(DataArray::from_vec(
            "Gradient Magnitude",
            gradient_mag,
            1,
        )));
    output
        .point_data_mut()
        .add_array(AnyDataArray::U8(DataArray::from_vec(
            "Classification",
            classification,
            1,
        )));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_grid_density() {
        let mut pd = PolyData::new();
        for i in 0..3 {
            for j in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }

        let result = point_cloud_density(&pd, 1.5);
        let arr = result.point_data().get_array("NeighborCount").unwrap();
        let mut buf = [0.0f64];
        // Center point (1,1) should have most neighbors
        arr.tuple_as_f64(4, &mut buf);
        assert!(buf[0] >= 5.0); // self + 4 cardinal neighbors at least
    }

    #[test]
    fn has_density_array() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let result = point_cloud_density(&pd, 2.0);
        assert!(result.point_data().get_array("Density").is_some());
        assert!(result.point_data().get_array("NeighborCount").is_some());
    }

    #[test]
    fn isolated_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([100.0, 0.0, 0.0]);

        let result = point_cloud_density(&pd, 1.0);
        let arr = result.point_data().get_array("NeighborCount").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0); // only self
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = point_cloud_density(&pd, 1.0);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn vtk_density_volume_defaults_shape() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let options = PointDensityOptions {
            sample_dimensions: [3, 3, 3],
            density_estimate: DensityEstimate::FixedRadius,
            density_form: DensityForm::NumberOfPoints,
            radius: 0.75,
            ..Default::default()
        };
        let result = point_density_filter(&pd, &options);
        assert_eq!(result.dimensions(), [3, 3, 3]);
        assert!(result.point_data().get_array("Density").is_some());
    }

    #[test]
    fn vtk_density_volume_gradient_arrays() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 1.0]);

        let options = PointDensityOptions {
            sample_dimensions: [3, 3, 3],
            density_estimate: DensityEstimate::FixedRadius,
            radius: 2.0,
            compute_gradient: true,
            ..Default::default()
        };
        let result = point_density_filter(&pd, &options);
        assert!(result.point_data().get_array("Gradient").is_some());
        assert!(result
            .point_data()
            .get_array("Gradient Magnitude")
            .is_some());
        assert!(result.point_data().get_array("Classification").is_some());
    }

    #[test]
    fn vtk_density_volume_uses_active_scalar_weights() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Weights",
                vec![2.0, 3.0],
                1,
            )));
        assert!(pd.point_data_mut().set_active_scalars("Weights"));

        let options = PointDensityOptions {
            sample_dimensions: [3, 3, 3],
            model_bounds: [-1.0, 2.0, -1.0, 1.0, -1.0, 1.0],
            density_estimate: DensityEstimate::FixedRadius,
            density_form: DensityForm::NumberOfPoints,
            radius: 5.0,
            scalar_weighting: true,
            ..Default::default()
        };
        let result = point_density_filter(&pd, &options);
        let arr = result.point_data().get_array("Density").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(13, &mut buf);
        assert_eq!(buf[0], 5.0);
    }
}
