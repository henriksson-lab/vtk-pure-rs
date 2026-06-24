//! Particle pathline computation over multiple time steps.
//!
//! Tracks individual particles through a sequence of vector fields,
//! recording each particle's full trajectory as a pathline.

use crate::data::{AnyDataArray, CellArray, DataArray, ImageData, Points, PolyData};

/// Compute particle pathlines through a time-varying vector field.
///
/// Unlike streaklines (which release new particles each step), pathlines
/// track the same set of particles from start to finish.
///
/// Returns a PolyData with one polyline per particle.
pub fn particle_pathlines(
    fields: &[&ImageData],
    initial_positions: &[[f64; 3]],
    step_size: f64,
    steps_per_field: usize,
) -> PolyData {
    if fields.is_empty() || initial_positions.is_empty() {
        return PolyData::new();
    }

    let n_particles = initial_positions.len();
    let mut positions: Vec<[f64; 3]> = initial_positions.to_vec();
    let mut alive: Vec<bool> = vec![true; n_particles];

    let mut out_points = Points::<f64>::new();
    let mut per_particle_ids: Vec<Vec<i64>> = vec![Vec::new(); n_particles];
    let mut time_data: Vec<f64> = Vec::new();
    let mut particle_id_data: Vec<f64> = Vec::new();
    let mut speed_data: Vec<f64> = Vec::new();

    for (fi, field) in fields.iter().enumerate() {
        let vectors = match field.point_data().vectors() {
            Some(v) if v.num_components() == 3 => v,
            _ => continue,
        };
        let dims = field.dimensions();
        let spacing = field.spacing();
        let origin = field.origin();

        for step in 0..steps_per_field {
            let t = fi as f64 + step as f64 / steps_per_field as f64;

            for pid in 0..n_particles {
                if !alive[pid] {
                    continue;
                }
                let pos = positions[pid];

                // Bounds check
                let in_bounds = (0..3).all(|i| {
                    pos[i] >= origin[i] && pos[i] <= origin[i] + (dims[i] as f64 - 1.0) * spacing[i]
                });
                if !in_bounds {
                    alive[pid] = false;
                    continue;
                }

                // Record point
                let idx = out_points.len() as i64;
                out_points.push(pos);
                time_data.push(t);
                particle_id_data.push(pid as f64);
                per_particle_ids[pid].push(idx);

                // RK4 integration
                let k1 = interp(vectors, pos, origin, spacing, dims);
                let spd = (k1[0] * k1[0] + k1[1] * k1[1] + k1[2] * k1[2]).sqrt();
                speed_data.push(spd);
                if spd < 1e-10 {
                    alive[pid] = false;
                    continue;
                }

                let p2 = [
                    pos[0] + 0.5 * step_size * k1[0],
                    pos[1] + 0.5 * step_size * k1[1],
                    pos[2] + 0.5 * step_size * k1[2],
                ];
                let k2 = interp(vectors, p2, origin, spacing, dims);
                let p3 = [
                    pos[0] + 0.5 * step_size * k2[0],
                    pos[1] + 0.5 * step_size * k2[1],
                    pos[2] + 0.5 * step_size * k2[2],
                ];
                let k3 = interp(vectors, p3, origin, spacing, dims);
                let p4 = [
                    pos[0] + step_size * k3[0],
                    pos[1] + step_size * k3[1],
                    pos[2] + step_size * k3[2],
                ];
                let k4 = interp(vectors, p4, origin, spacing, dims);

                positions[pid] = [
                    pos[0] + step_size / 6.0 * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]),
                    pos[1] + step_size / 6.0 * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]),
                    pos[2] + step_size / 6.0 * (k1[2] + 2.0 * k2[2] + 2.0 * k3[2] + k4[2]),
                ];
            }
        }
    }

    let mut lines = CellArray::new();
    for ids in &per_particle_ids {
        if ids.len() >= 2 {
            lines.push_cell(ids);
        }
    }

    let mut result = PolyData::new();
    result.points = out_points;
    result.lines = lines;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ParticleId",
            particle_id_data,
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Time", time_data, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Speed", speed_data, 1,
        )));
    result
}

fn interp(
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
    let mut r = [0.0; 3];
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
                        r[c] += w * buf[c];
                    }
                }
            }
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_field() -> ImageData {
        let dims = [10, 10, 10];
        let n = dims[0] * dims[1] * dims[2];
        let mut v = Vec::with_capacity(n * 3);
        for _ in 0..n {
            v.push(1.0);
            v.push(0.0);
            v.push(0.0);
        }
        let mut f = ImageData::with_dimensions(dims[0], dims[1], dims[2]);
        f.set_spacing([1.0, 1.0, 1.0]);
        f.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("vel", v, 3)));
        f.point_data_mut().set_active_vectors("vel");
        f
    }

    #[test]
    fn basic_pathlines() {
        let field = make_field();
        let result = particle_pathlines(&[&field, &field], &[[2.0, 5.0, 5.0]], 0.1, 10);
        assert!(result.points.len() > 5);
        assert!(result.lines.num_cells() >= 1);
        assert!(result.point_data().get_array("Speed").is_some());
    }

    #[test]
    fn multiple_particles() {
        let field = make_field();
        let result = particle_pathlines(&[&field], &[[2.0, 3.0, 5.0], [2.0, 7.0, 5.0]], 0.1, 10);
        assert!(result.lines.num_cells() >= 2);
    }

    #[test]
    fn empty() {
        let result = particle_pathlines(&[], &[[0.0, 0.0, 0.0]], 0.1, 10);
        assert_eq!(result.points.len(), 0);
    }
}
