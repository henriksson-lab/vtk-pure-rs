//! Time-dependent particle advection through vector fields.
//!
//! Unlike `stream_tracer` which computes streamlines in a steady field,
//! `particle_tracer` advects particles through a time-varying vector field,
//! producing pathlines.

use crate::data::{AnyDataArray, CellArray, DataArray, ImageData, Points, PolyData};

/// Parameters for particle tracing.
pub struct ParticleTracerParams {
    /// Maximum number of integration steps per time step. Default: 100
    pub max_steps: usize,
    /// Integration step size. Default: 0.01
    pub step_size: f64,
    /// Minimum velocity magnitude to continue. Default: 1e-8
    pub terminal_speed: f64,
}

impl Default for ParticleTracerParams {
    fn default() -> Self {
        Self {
            max_steps: 100,
            step_size: 0.01,
            terminal_speed: 1e-8,
        }
    }
}

/// Advect particles through a steady vector field on ImageData, producing pathlines.
///
/// Each seed point is advected using RK4 integration with trilinear interpolation.
/// Returns polylines where each line is one particle's path.
pub fn particle_trace_steady(
    field: &ImageData,
    seeds: &[[f64; 3]],
    params: &ParticleTracerParams,
) -> PolyData {
    let vectors = match field.point_data().vectors() {
        Some(v) if v.num_components() == 3 => v,
        _ => return PolyData::new(),
    };

    let dims = field.dimensions();
    let spacing = field.spacing();
    let origin = field.origin();

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut particle_id_data: Vec<f64> = Vec::new();

    for (pid, seed) in seeds.iter().enumerate() {
        let mut pos = *seed;
        let mut line_ids: Vec<i64> = Vec::new();

        for _ in 0..params.max_steps {
            // Check bounds
            if !in_bounds(pos, origin, spacing, dims) {
                break;
            }

            let idx = out_points.len() as i64;
            out_points.push(pos);
            particle_id_data.push(pid as f64);
            line_ids.push(idx);

            // RK4 integration
            let k1 = interpolate_vector_field(vectors, pos, origin, spacing, dims);
            let speed = (k1[0] * k1[0] + k1[1] * k1[1] + k1[2] * k1[2]).sqrt();
            if speed < params.terminal_speed {
                break;
            }

            let p2 = [
                pos[0] + 0.5 * params.step_size * k1[0],
                pos[1] + 0.5 * params.step_size * k1[1],
                pos[2] + 0.5 * params.step_size * k1[2],
            ];
            let k2 = interpolate_vector_field(vectors, p2, origin, spacing, dims);

            let p3 = [
                pos[0] + 0.5 * params.step_size * k2[0],
                pos[1] + 0.5 * params.step_size * k2[1],
                pos[2] + 0.5 * params.step_size * k2[2],
            ];
            let k3 = interpolate_vector_field(vectors, p3, origin, spacing, dims);

            let p4 = [
                pos[0] + params.step_size * k3[0],
                pos[1] + params.step_size * k3[1],
                pos[2] + params.step_size * k3[2],
            ];
            let k4 = interpolate_vector_field(vectors, p4, origin, spacing, dims);

            pos = [
                pos[0] + params.step_size / 6.0 * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]),
                pos[1] + params.step_size / 6.0 * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]),
                pos[2] + params.step_size / 6.0 * (k1[2] + 2.0 * k2[2] + 2.0 * k3[2] + k4[2]),
            ];
        }

        if line_ids.len() >= 2 {
            out_lines.push_cell(&line_ids);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = out_points;
    mesh.lines = out_lines;
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ParticleId",
            particle_id_data,
            1,
        )));
    mesh
}

/// Advect particles through a time-varying field (sequence of ImageData).
///
/// Each ImageData in `fields` represents the vector field at one time step.
/// Particles are advected for `steps_per_field` integration steps per field,
/// then the field advances to the next time step.
pub fn particle_trace_temporal(
    fields: &[&ImageData],
    seeds: &[[f64; 3]],
    steps_per_field: usize,
    step_size: f64,
) -> PolyData {
    if fields.is_empty() {
        return PolyData::new();
    }

    let dims = fields[0].dimensions();
    let spacing = fields[0].spacing();
    let origin = fields[0].origin();

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut time_data: Vec<f64> = Vec::new();
    let mut particle_id_data: Vec<f64> = Vec::new();

    // Current particle positions
    let mut positions: Vec<[f64; 3]> = seeds.to_vec();
    let mut alive: Vec<bool> = vec![true; seeds.len()];
    let mut line_ids: Vec<Vec<i64>> = vec![Vec::new(); seeds.len()];

    for (fi, field) in fields.iter().enumerate() {
        let vectors = match field.point_data().vectors() {
            Some(v) if v.num_components() == 3 => v,
            _ => continue,
        };

        let t_base = fi as f64;

        for step in 0..steps_per_field {
            let t = t_base + step as f64 / steps_per_field as f64;

            for (pid, pos) in positions.iter_mut().enumerate() {
                if !alive[pid] {
                    continue;
                }

                if !in_bounds(*pos, origin, spacing, dims) {
                    alive[pid] = false;
                    continue;
                }

                let idx = out_points.len() as i64;
                out_points.push(*pos);
                time_data.push(t);
                particle_id_data.push(pid as f64);
                line_ids[pid].push(idx);

                // RK4 step
                let k1 = interpolate_vector_field(vectors, *pos, origin, spacing, dims);
                let speed = (k1[0] * k1[0] + k1[1] * k1[1] + k1[2] * k1[2]).sqrt();
                if speed < 1e-8 {
                    alive[pid] = false;
                    continue;
                }

                let p2 = [
                    pos[0] + 0.5 * step_size * k1[0],
                    pos[1] + 0.5 * step_size * k1[1],
                    pos[2] + 0.5 * step_size * k1[2],
                ];
                let k2 = interpolate_vector_field(vectors, p2, origin, spacing, dims);
                let p3 = [
                    pos[0] + 0.5 * step_size * k2[0],
                    pos[1] + 0.5 * step_size * k2[1],
                    pos[2] + 0.5 * step_size * k2[2],
                ];
                let k3 = interpolate_vector_field(vectors, p3, origin, spacing, dims);
                let p4 = [
                    pos[0] + step_size * k3[0],
                    pos[1] + step_size * k3[1],
                    pos[2] + step_size * k3[2],
                ];
                let k4 = interpolate_vector_field(vectors, p4, origin, spacing, dims);

                *pos = [
                    pos[0] + step_size / 6.0 * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]),
                    pos[1] + step_size / 6.0 * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]),
                    pos[2] + step_size / 6.0 * (k1[2] + 2.0 * k2[2] + 2.0 * k3[2] + k4[2]),
                ];
            }
        }
    }

    // Build output lines
    for ids in &line_ids {
        if ids.len() >= 2 {
            out_lines.push_cell(ids);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = out_points;
    mesh.lines = out_lines;
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ParticleId",
            particle_id_data,
            1,
        )));
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Time", time_data, 1)));
    mesh
}

fn in_bounds(pos: [f64; 3], origin: [f64; 3], spacing: [f64; 3], dims: [usize; 3]) -> bool {
    for i in 0..3 {
        let max = origin[i] + (dims[i] as f64 - 1.0) * spacing[i];
        if pos[i] < origin[i] || pos[i] > max {
            return false;
        }
    }
    true
}

fn interpolate_vector_field(
    vectors: &AnyDataArray,
    pos: [f64; 3],
    origin: [f64; 3],
    spacing: [f64; 3],
    dims: [usize; 3],
) -> [f64; 3] {
    // Compute fractional grid coordinates
    let fx = (pos[0] - origin[0]) / spacing[0];
    let fy = (pos[1] - origin[1]) / spacing[1];
    let fz = (pos[2] - origin[2]) / spacing[2];

    let ix = fx.floor() as usize;
    let iy = fy.floor() as usize;
    let iz = fz.floor() as usize;

    let ix = ix.min(dims[0].saturating_sub(2));
    let iy = iy.min(dims[1].saturating_sub(2));
    let iz = iz.min(dims[2].saturating_sub(2));

    let tx = fx - ix as f64;
    let ty = fy - iy as f64;
    let tz = fz - iz as f64;

    let tx = tx.clamp(0.0, 1.0);
    let ty = ty.clamp(0.0, 1.0);
    let tz = tz.clamp(0.0, 1.0);

    // Trilinear interpolation
    let mut result = [0.0; 3];
    let mut buf = [0.0f64; 3];

    for dz in 0..2usize {
        for dy in 0..2usize {
            for dx in 0..2usize {
                let idx = (ix + dx) + (iy + dy) * dims[0] + (iz + dz) * dims[0] * dims[1];
                if idx < vectors.num_tuples() {
                    vectors.tuple_as_f64(idx, &mut buf);
                    let wx = if dx == 0 { 1.0 - tx } else { tx };
                    let wy = if dy == 0 { 1.0 - ty } else { ty };
                    let wz = if dz == 0 { 1.0 - tz } else { tz };
                    let w = wx * wy * wz;
                    result[0] += w * buf[0];
                    result[1] += w * buf[1];
                    result[2] += w * buf[2];
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_uniform_field() -> ImageData {
        let dims = [10, 10, 10];
        let spacing = [1.0, 1.0, 1.0];
        let origin = [0.0, 0.0, 0.0];
        let n = dims[0] * dims[1] * dims[2];

        let mut vx = Vec::with_capacity(n * 3);
        for _ in 0..n {
            vx.push(1.0); // uniform flow in +X
            vx.push(0.0);
            vx.push(0.0);
        }

        let mut field = ImageData::new();
        field.set_extent([
            0,
            dims[0] as i64 - 1,
            0,
            dims[1] as i64 - 1,
            0,
            dims[2] as i64 - 1,
        ]);
        field.set_spacing(spacing);
        field.set_origin(origin);
        field
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("velocity", vx, 3)));
        field.point_data_mut().set_active_vectors("velocity");
        field
    }

    #[test]
    fn steady_uniform_flow() {
        let field = make_uniform_field();
        let seeds = vec![[1.0, 5.0, 5.0]];
        let params = ParticleTracerParams {
            max_steps: 50,
            step_size: 0.1,
            ..Default::default()
        };

        let result = particle_trace_steady(&field, &seeds, &params);
        assert!(result.points.len() > 10);
        assert_eq!(result.lines.num_cells(), 1);

        // Particle should have moved in +X
        let last = result.points.get(result.points.len() - 1);
        assert!(last[0] > 1.5); // moved right
        assert!((last[1] - 5.0).abs() < 0.01); // stayed in Y
    }

    #[test]
    fn temporal_advection() {
        let field = make_uniform_field();
        let seeds = vec![[2.0, 5.0, 5.0], [3.0, 5.0, 5.0]];

        let result = particle_trace_temporal(&[&field, &field], &seeds, 10, 0.1);
        assert!(result.points.len() > 10);
        assert!(result.lines.num_cells() >= 1);
        assert!(result.point_data().get_array("Time").is_some());
        assert!(result.point_data().get_array("ParticleId").is_some());
    }

    #[test]
    fn empty_field() {
        let field = ImageData::new();
        let seeds = vec![[0.0, 0.0, 0.0]];
        let result = particle_trace_steady(&field, &seeds, &ParticleTracerParams::default());
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn out_of_bounds_terminates() {
        let field = make_uniform_field();
        let seeds = vec![[8.5, 5.0, 5.0]]; // near boundary, will exit quickly
        let params = ParticleTracerParams {
            max_steps: 1000,
            step_size: 0.5,
            ..Default::default()
        };

        let result = particle_trace_steady(&field, &seeds, &params);
        // Should terminate before max_steps due to leaving bounds
        assert!(result.points.len() < 100);
    }
}
