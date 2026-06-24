//! Lagrangian particle tracking with properties.
//!
//! Tracks particles through a vector field, recording per-particle
//! properties like age, distance traveled, and accumulated scalars.

use crate::data::{AnyDataArray, CellArray, DataArray, ImageData, Points, PolyData};

/// A tracked particle with accumulated properties.
#[derive(Debug, Clone)]
pub struct TrackedParticle {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub age: f64,
    pub distance_traveled: f64,
    pub alive: bool,
    pub id: usize,
}

/// Track particles through a vector field with property accumulation.
///
/// Returns a PolyData with pathlines and per-point Age, Distance, Speed arrays.
pub fn lagrangian_track(
    field: &ImageData,
    seeds: &[[f64; 3]],
    dt: f64,
    max_steps: usize,
    max_age: f64,
) -> PolyData {
    let vectors = match field.point_data().vectors() {
        Some(v) if v.num_components() == 3 => v,
        _ => return PolyData::new(),
    };
    let dims = field.dimensions();
    let spacing = field.spacing();
    let origin = field.origin();

    let mut particles: Vec<TrackedParticle> = seeds
        .iter()
        .enumerate()
        .map(|(i, &pos)| TrackedParticle {
            position: pos,
            velocity: [0.0; 3],
            age: 0.0,
            distance_traveled: 0.0,
            alive: true,
            id: i,
        })
        .collect();

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut age_data = Vec::new();
    let mut dist_data = Vec::new();
    let mut speed_data = Vec::new();
    let mut pid_data = Vec::new();
    let mut per_particle_ids: Vec<Vec<i64>> = vec![Vec::new(); seeds.len()];

    for _ in 0..max_steps {
        let any_alive = particles.iter().any(|p| p.alive);
        if !any_alive {
            break;
        }

        for p in &mut particles {
            if !p.alive {
                continue;
            }

            // Bounds check
            let in_bounds = (0..3).all(|i| {
                p.position[i] >= origin[i]
                    && p.position[i] <= origin[i] + (dims[i] as f64 - 1.0) * spacing[i]
            });
            if !in_bounds || p.age > max_age {
                p.alive = false;
                continue;
            }

            // Record
            let idx = out_points.len() as i64;
            out_points.push(p.position);
            age_data.push(p.age);
            dist_data.push(p.distance_traveled);
            pid_data.push(p.id as f64);
            per_particle_ids[p.id].push(idx);

            // Get velocity
            let v = interp_vec(vectors, p.position, origin, spacing, dims);
            let spd = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            speed_data.push(spd);
            p.velocity = v;

            if spd < 1e-10 {
                p.alive = false;
                continue;
            }

            // Euler step (simple for Lagrangian tracking)
            let new_pos = [
                p.position[0] + dt * v[0],
                p.position[1] + dt * v[1],
                p.position[2] + dt * v[2],
            ];
            let dx = new_pos[0] - p.position[0];
            let dy = new_pos[1] - p.position[1];
            let dz = new_pos[2] - p.position[2];
            p.distance_traveled += (dx * dx + dy * dy + dz * dz).sqrt();
            p.position = new_pos;
            p.age += dt;
        }
    }

    for ids in &per_particle_ids {
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
            "ParticleId",
            pid_data,
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Age", age_data, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Distance", dist_data, 1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Speed", speed_data, 1,
        )));
    result
}

fn interp_vec(
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
    fn basic_tracking() {
        let field = make_field();
        let result = lagrangian_track(&field, &[[2.0, 5.0, 5.0]], 0.1, 50, 100.0);
        assert!(result.points.len() > 5);
        assert!(result.point_data().get_array("Age").is_some());
        assert!(result.point_data().get_array("Distance").is_some());
    }

    #[test]
    fn age_limit() {
        let field = make_field();
        let result = lagrangian_track(&field, &[[2.0, 5.0, 5.0]], 0.1, 1000, 1.0);
        // Should stop early due to age limit
        let arr = result.point_data().get_array("Age").unwrap();
        let mut buf = [0.0f64];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0] <= 1.1);
        }
    }

    #[test]
    fn empty() {
        let field = ImageData::new();
        let result = lagrangian_track(&field, &[[0.0, 0.0, 0.0]], 0.1, 10, 10.0);
        assert_eq!(result.points.len(), 0);
    }
}
