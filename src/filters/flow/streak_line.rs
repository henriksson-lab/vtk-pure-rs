//! Streakline computation for unsteady vector fields.
//!
//! A streakline is the locus of all particles that have passed through
//! a particular seed point at any previous time step. This is computed
//! by releasing new particles at the seed at each time step and advecting
//! all living particles.

use crate::data::{AnyDataArray, CellArray, DataArray, ImageData, Points, PolyData};

/// Compute streaklines for a time-varying vector field.
///
/// At each time step, a new particle is released from each seed point.
/// All living particles are advected through the current field.
/// The resulting polylines connect particles from the same seed.
///
/// `fields` — sequence of ImageData with vector point data (one per time step)
/// `seeds` — seed points where particles are released
/// `steps_per_field` — integration sub-steps per time step
/// `step_size` — integration step size
pub fn streak_lines(
    fields: &[&ImageData],
    seeds: &[[f64; 3]],
    steps_per_field: usize,
    step_size: f64,
) -> PolyData {
    if fields.is_empty() || seeds.is_empty() {
        return PolyData::new();
    }

    let dims = fields[0].dimensions();
    let spacing = fields[0].spacing();
    let origin = fields[0].origin();

    struct Particle {
        pos: [f64; 3],
        seed_idx: usize,
        _birth_time: usize,
        alive: bool,
    }

    let mut all_particles: Vec<Particle> = Vec::new();
    let mut out_points = Points::<f64>::new();
    let mut seed_id_data: Vec<f64> = Vec::new();
    let mut time_data: Vec<f64> = Vec::new();

    // Per-seed: collect point indices for building polylines
    let mut seed_point_indices: Vec<Vec<i64>> = vec![Vec::new(); seeds.len()];

    for (fi, field) in fields.iter().enumerate() {
        let vectors = match field.point_data().vectors() {
            Some(v) if v.num_components() == 3 => v,
            _ => continue,
        };

        // Release new particles at each seed
        for (si, seed) in seeds.iter().enumerate() {
            all_particles.push(Particle {
                pos: *seed,
                seed_idx: si,
                _birth_time: fi,
                alive: true,
            });
        }

        // Advect all particles
        for _ in 0..steps_per_field {
            for particle in all_particles.iter_mut() {
                if !particle.alive {
                    continue;
                }

                // Check bounds
                if !in_bounds(particle.pos, origin, spacing, dims) {
                    particle.alive = false;
                    continue;
                }

                // Record position
                let idx = out_points.len() as i64;
                out_points.push(particle.pos);
                seed_id_data.push(particle.seed_idx as f64);
                time_data.push(fi as f64);
                seed_point_indices[particle.seed_idx].push(idx);

                // RK2 advection
                let v1 = interp_vec3(vectors, particle.pos, origin, spacing, dims);
                let speed = (v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]).sqrt();
                if speed < 1e-8 {
                    particle.alive = false;
                    continue;
                }

                let mid = [
                    particle.pos[0] + 0.5 * step_size * v1[0],
                    particle.pos[1] + 0.5 * step_size * v1[1],
                    particle.pos[2] + 0.5 * step_size * v1[2],
                ];
                let v2 = interp_vec3(vectors, mid, origin, spacing, dims);

                particle.pos = [
                    particle.pos[0] + step_size * v2[0],
                    particle.pos[1] + step_size * v2[1],
                    particle.pos[2] + step_size * v2[2],
                ];
            }
        }
    }

    // Build polylines per seed
    let mut out_lines = CellArray::new();
    for ids in &seed_point_indices {
        if ids.len() >= 2 {
            out_lines.push_cell(ids);
        }
    }

    let mut result = PolyData::new();
    result.points = out_points;
    result.lines = out_lines;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SeedId",
            seed_id_data,
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Time", time_data, 1)));
    result
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

fn interp_vec3(
    vectors: &AnyDataArray,
    pos: [f64; 3],
    origin: [f64; 3],
    spacing: [f64; 3],
    dims: [usize; 3],
) -> [f64; 3] {
    let fx = (pos[0] - origin[0]) / spacing[0];
    let fy = (pos[1] - origin[1]) / spacing[1];
    let fz = (pos[2] - origin[2]) / spacing[2];
    let ix = (fx.floor() as usize).min(dims[0].saturating_sub(2));
    let iy = (fy.floor() as usize).min(dims[1].saturating_sub(2));
    let iz = (fz.floor() as usize).min(dims[2].saturating_sub(2));
    let tx = (fx - ix as f64).clamp(0.0, 1.0);
    let ty = (fy - iy as f64).clamp(0.0, 1.0);
    let tz = (fz - iz as f64).clamp(0.0, 1.0);

    let mut result = [0.0; 3];
    let mut buf = [0.0f64; 3];
    for dz in 0..2usize {
        for dy in 0..2usize {
            for dx in 0..2usize {
                let idx = (ix + dx) + (iy + dy) * dims[0] + (iz + dz) * dims[0] * dims[1];
                if idx < vectors.num_tuples() {
                    vectors.tuple_as_f64(idx, &mut buf);
                    let w = (if dx == 0 { 1.0 - tx } else { tx })
                        * (if dy == 0 { 1.0 - ty } else { ty })
                        * (if dz == 0 { 1.0 - tz } else { tz });
                    for c in 0..3 {
                        result[c] += w * buf[c];
                    }
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
        let n = dims[0] * dims[1] * dims[2];
        let mut vdata = Vec::with_capacity(n * 3);
        for _ in 0..n {
            vdata.push(1.0);
            vdata.push(0.0);
            vdata.push(0.0);
        }
        let mut field = ImageData::with_dimensions(dims[0], dims[1], dims[2]);
        field.set_spacing([1.0, 1.0, 1.0]);
        field.set_origin([0.0, 0.0, 0.0]);
        field
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("velocity", vdata, 3)));
        field.point_data_mut().set_active_vectors("velocity");
        field
    }

    #[test]
    fn basic_streakline() {
        let field = make_uniform_field();
        let seeds = vec![[2.0, 5.0, 5.0]];
        let result = streak_lines(&[&field, &field, &field], &seeds, 5, 0.1);
        assert!(result.points.len() > 5);
        assert!(result.lines.num_cells() >= 1);
        assert!(result.point_data().get_array("SeedId").is_some());
        assert!(result.point_data().get_array("Time").is_some());
    }

    #[test]
    fn multiple_seeds() {
        let field = make_uniform_field();
        let seeds = vec![[2.0, 3.0, 5.0], [2.0, 7.0, 5.0]];
        let result = streak_lines(&[&field, &field], &seeds, 3, 0.1);
        assert!(result.lines.num_cells() >= 2);
    }

    #[test]
    fn empty_inputs() {
        let result = streak_lines(&[], &[[0.0, 0.0, 0.0]], 5, 0.1);
        assert_eq!(result.points.len(), 0);
    }
}
