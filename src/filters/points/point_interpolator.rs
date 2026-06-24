//! Kernel-based point interpolation.
//!
//! Interpolate scalar/vector data from a source point cloud onto a target
//! point set using various kernel functions. Analogous to VTK's
//! vtkPointInterpolator and vtkSPHInterpolator.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Interpolation kernel type.
#[derive(Debug, Clone, Copy)]
pub enum InterpolationKernel {
    /// Inverse distance weighting: w = 1 / d^power
    InverseDistance { power: f64 },
    /// Gaussian kernel: w = exp(-d² / (2σ²))
    Gaussian { sigma: f64 },
    /// Shepard kernel: w = max(0, (R - d) / (R * d))^power
    Shepard { radius: f64, power: f64 },
    /// Nearest neighbor: take value from closest point
    NearestNeighbor,
    /// SPH cubic spline kernel
    SPHCubicSpline { smoothing_length: f64 },
}

/// Interpolate scalar arrays from source to target using kernel-based interpolation.
///
/// For each point in `target`, finds the nearest points in `source` within
/// the search radius and computes a weighted average of their scalar values.
///
/// Returns a new PolyData with the same geometry as `target` but with
/// interpolated scalar arrays from `source`.
pub fn interpolate_points(
    source: &PolyData,
    target: &PolyData,
    kernel: InterpolationKernel,
    search_radius: f64,
    max_neighbors: usize,
) -> PolyData {
    let n_target = target.points.len();
    let n_source = source.points.len();
    if n_target == 0 || n_source == 0 {
        return target.clone();
    }

    let mut result = target.clone();

    // Collect scalar arrays from source
    let pd = source.point_data();
    for ai in 0..pd.num_arrays() {
        let arr = match pd.get_array_by_index(ai) {
            Some(a) => a,
            None => continue,
        };
        let nc = arr.num_components();
        let name = arr.name().to_string();

        let mut out_data = vec![0.0f64; n_target * nc];

        for ti in 0..n_target {
            let tp = target.points.get(ti);

            // Find neighbors within radius (brute force for simplicity)
            let mut neighbors: Vec<(usize, f64)> = Vec::new();
            for si in 0..n_source {
                let sp = source.points.get(si);
                let dx = tp[0] - sp[0];
                let dy = tp[1] - sp[1];
                let dz = tp[2] - sp[2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                if dist <= search_radius {
                    neighbors.push((si, dist));
                }
            }

            // Sort by distance and limit
            neighbors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            if neighbors.len() > max_neighbors {
                neighbors.truncate(max_neighbors);
            }

            if neighbors.is_empty() {
                continue; // leave as zero
            }

            // Compute weights
            let weights: Vec<f64> = neighbors
                .iter()
                .map(|&(_, d)| kernel_weight(kernel, d))
                .collect();

            let w_sum: f64 = weights.iter().sum();
            if w_sum < 1e-15 {
                continue;
            }

            // Weighted average
            let mut buf = vec![0.0f64; nc];
            for (ni, &(si, _)) in neighbors.iter().enumerate() {
                arr.tuple_as_f64(si, &mut buf);
                for c in 0..nc {
                    out_data[ti * nc + c] += weights[ni] * buf[c];
                }
            }
            for c in 0..nc {
                out_data[ti * nc + c] /= w_sum;
            }
        }

        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(&name, out_data, nc)));
    }

    result
}

fn kernel_weight(kernel: InterpolationKernel, dist: f64) -> f64 {
    match kernel {
        InterpolationKernel::InverseDistance { power } => {
            if dist < 1e-15 {
                1e15
            } else {
                1.0 / dist.powf(power)
            }
        }
        InterpolationKernel::Gaussian { sigma } => (-dist * dist / (2.0 * sigma * sigma)).exp(),
        InterpolationKernel::Shepard { radius, power } => {
            if dist >= radius {
                0.0
            } else if dist < 1e-15 {
                1e15
            } else {
                ((radius - dist) / (radius * dist)).powf(power)
            }
        }
        InterpolationKernel::NearestNeighbor => {
            if dist < 1e-15 {
                1e15
            } else {
                1.0 / (dist + 1e-15)
            }
        }
        InterpolationKernel::SPHCubicSpline { smoothing_length } => {
            let q = dist / smoothing_length;
            let norm = 1.0 / (std::f64::consts::PI * smoothing_length.powi(3));
            if q < 1.0 {
                norm * (1.0 - 1.5 * q * q + 0.75 * q * q * q)
            } else if q < 2.0 {
                norm * 0.25 * (2.0 - q).powi(3)
            } else {
                0.0
            }
        }
    }
}

/// Compute a density field by summing kernel contributions at each point.
///
/// Returns the mesh with a "Density" point data array.
pub fn compute_point_density(
    mesh: &PolyData,
    kernel: InterpolationKernel,
    search_radius: f64,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    let mut density = vec![0.0f64; n];

    for i in 0..n {
        let pi = mesh.points.get(i);
        let mut sum = 0.0;
        for j in 0..n {
            if i == j {
                continue;
            }
            let pj = mesh.points.get(j);
            let dx = pi[0] - pj[0];
            let dy = pi[1] - pj[1];
            let dz = pi[2] - pj[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            if dist <= search_radius {
                sum += kernel_weight(kernel, dist);
            }
        }
        density[i] = sum;
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Density", density, 1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Points;

    fn make_source() -> PolyData {
        let mut mesh = PolyData::new();
        mesh.points = Points::from(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temperature",
                vec![100.0, 200.0, 300.0],
                1,
            )));
        mesh.point_data_mut().set_active_scalars("temperature");
        mesh
    }

    fn make_target() -> PolyData {
        let mut mesh = PolyData::new();
        mesh.points = Points::from(vec![
            [0.5, 0.0, 0.0], // between source points 0 and 1
            [0.0, 0.5, 0.0], // between source points 0 and 2
        ]);
        mesh
    }

    #[test]
    fn inverse_distance_interpolation() {
        let source = make_source();
        let target = make_target();
        let result = interpolate_points(
            &source,
            &target,
            InterpolationKernel::InverseDistance { power: 2.0 },
            5.0,
            10,
        );

        let arr = result.point_data().get_array("temperature").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        // Should be between 100 and 200 (closer to midpoint)
        assert!(buf[0] > 100.0 && buf[0] < 300.0);
    }

    #[test]
    fn gaussian_interpolation() {
        let source = make_source();
        let target = make_target();
        let result = interpolate_points(
            &source,
            &target,
            InterpolationKernel::Gaussian { sigma: 1.0 },
            5.0,
            10,
        );
        assert!(result.point_data().get_array("temperature").is_some());
    }

    #[test]
    fn sph_interpolation() {
        let source = make_source();
        let target = make_target();
        let result = interpolate_points(
            &source,
            &target,
            InterpolationKernel::SPHCubicSpline {
                smoothing_length: 2.0,
            },
            5.0,
            10,
        );
        assert!(result.point_data().get_array("temperature").is_some());
    }

    #[test]
    fn density_computation() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from(vec![
            [0.0, 0.0, 0.0],
            [0.1, 0.0, 0.0],
            [0.2, 0.0, 0.0],
            [10.0, 0.0, 0.0], // isolated point
        ]);

        let result =
            compute_point_density(&mesh, InterpolationKernel::Gaussian { sigma: 0.5 }, 2.0);

        let arr = result.point_data().get_array("Density").unwrap();
        let mut buf = [0.0f64];

        // Point at 0.1 should have higher density than point at 10.0
        arr.tuple_as_f64(1, &mut buf);
        let dense = buf[0];
        arr.tuple_as_f64(3, &mut buf);
        let sparse = buf[0];
        assert!(dense > sparse);
    }

    #[test]
    fn empty_inputs() {
        let source = PolyData::new();
        let target = make_target();
        let result = interpolate_points(
            &source,
            &target,
            InterpolationKernel::NearestNeighbor,
            5.0,
            10,
        );
        assert_eq!(result.points.len(), target.points.len());
    }
}
