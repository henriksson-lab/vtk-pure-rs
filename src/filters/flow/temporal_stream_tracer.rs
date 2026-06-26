//! Time-varying streamline integration.
//!
//! Integrates streamlines through a sequence of vector fields, using
//! temporal interpolation between time steps for smooth trajectories.

use crate::data::{AnyDataArray, CellArray, DataArray, ImageData, Points, PolyData};

/// Integrate streamlines through a time-varying vector field with
/// temporal interpolation between consecutive fields.
///
/// At each integration step, the velocity is linearly interpolated
/// between `fields[t]` and `fields[t+1]` based on the fractional time.
pub fn temporal_stream_trace(
    fields: &[&ImageData],
    seeds: &[[f64; 3]],
    step_size: f64,
    max_steps: usize,
) -> PolyData {
    if fields.len() < 2 || seeds.is_empty() || step_size <= 0.0 {
        return PolyData::new();
    }

    let n_fields = fields.len();
    let total_time = (n_fields - 1) as f64;

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut time_data: Vec<f64> = Vec::new();

    for seed in seeds {
        let mut pos = *seed;
        let mut t = 0.0f64;
        let mut line_ids: Vec<i64> = Vec::new();

        for _ in 0..max_steps {
            if t >= total_time {
                break;
            }

            // Temporal interpolation
            let fi = (t.floor() as usize).min(n_fields - 2);
            let frac = t - fi as f64;
            let dims0 = fields[fi].dimensions();
            let extent0 = fields[fi].extent();
            let spacing0 = fields[fi].spacing();
            let origin0 = fields[fi].origin();
            let dims1 = fields[fi + 1].dimensions();
            let extent1 = fields[fi + 1].extent();
            let spacing1 = fields[fi + 1].spacing();
            let origin1 = fields[fi + 1].origin();

            if !in_bounds(pos, origin0, spacing0, extent0)
                || !in_bounds(pos, origin1, spacing1, extent1)
            {
                break;
            }

            let v0_arr = match fields[fi].point_data().vectors() {
                Some(v) if v.num_components() == 3 => v,
                _ => break,
            };
            let v1_arr = match fields[fi + 1].point_data().vectors() {
                Some(v) if v.num_components() == 3 => v,
                _ => break,
            };

            let vel0 = interp_spatial(v0_arr, pos, origin0, spacing0, extent0, dims0);
            let vel1 = interp_spatial(v1_arr, pos, origin1, spacing1, extent1, dims1);

            let vel = [
                vel0[0] * (1.0 - frac) + vel1[0] * frac,
                vel0[1] * (1.0 - frac) + vel1[1] * frac,
                vel0[2] * (1.0 - frac) + vel1[2] * frac,
            ];

            let speed = (vel[0] * vel[0] + vel[1] * vel[1] + vel[2] * vel[2]).sqrt();
            if speed < 1e-10 {
                break;
            }

            let idx = out_points.len() as i64;
            out_points.push(pos);
            time_data.push(t);
            line_ids.push(idx);

            pos = [
                pos[0] + step_size * vel[0],
                pos[1] + step_size * vel[1],
                pos[2] + step_size * vel[2],
            ];
            t += step_size * speed.recip().min(1.0); // advance time proportionally
            t = t.min(total_time);
        }

        if line_ids.len() >= 2 {
            out_lines.push_cell(&line_ids);
        }
    }

    let mut result = PolyData::new();
    result.points = out_points;
    result.lines = out_lines;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "IntegrationTime",
            time_data,
            1,
        )));
    result
}

fn in_bounds(pos: [f64; 3], origin: [f64; 3], spacing: [f64; 3], extent: [i64; 6]) -> bool {
    if spacing.iter().any(|&s| s == 0.0) {
        return false;
    }
    (0..3).all(|i| {
        let lo = origin[i] + extent[2 * i] as f64 * spacing[i];
        let hi = origin[i] + extent[2 * i + 1] as f64 * spacing[i];
        pos[i] >= lo.min(hi) && pos[i] <= lo.max(hi)
    })
}

fn interp_spatial(
    arr: &AnyDataArray,
    pos: [f64; 3],
    origin: [f64; 3],
    spacing: [f64; 3],
    extent: [i64; 6],
    dims: [usize; 3],
) -> [f64; 3] {
    if dims.iter().any(|&d| d == 0) || spacing.iter().any(|&s| s == 0.0) {
        return [0.0; 3];
    }

    let fx = (pos[0] - origin[0]) / spacing[0] - extent[0] as f64;
    let fy = (pos[1] - origin[1]) / spacing[1] - extent[2] as f64;
    let fz = (pos[2] - origin[2]) / spacing[2] - extent[4] as f64;
    if fx < 0.0
        || fy < 0.0
        || fz < 0.0
        || fx > (dims[0] - 1) as f64
        || fy > (dims[1] - 1) as f64
        || fz > (dims[2] - 1) as f64
    {
        return [0.0; 3];
    }

    let ix = if dims[0] > 1 {
        (fx.floor() as usize).min(dims[0] - 2)
    } else {
        0
    };
    let iy = if dims[1] > 1 {
        (fy.floor() as usize).min(dims[1] - 2)
    } else {
        0
    };
    let iz = if dims[2] > 1 {
        (fz.floor() as usize).min(dims[2] - 2)
    } else {
        0
    };
    let tx = if dims[0] > 1 {
        (fx - ix as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let ty = if dims[1] > 1 {
        (fy - iy as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let tz = if dims[2] > 1 {
        (fz - iz as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut r = [0.0; 3];
    let mut buf = [0.0f64; 3];
    let nz = if dims[2] > 1 { 2 } else { 1 };
    let ny = if dims[1] > 1 { 2 } else { 1 };
    let nx = if dims[0] > 1 { 2 } else { 1 };
    for dz in 0..nz {
        for dy in 0..ny {
            for dx in 0..nx {
                let idx = (ix + dx) + (iy + dy) * dims[0] + (iz + dz) * dims[0] * dims[1];
                if idx < arr.num_tuples() {
                    arr.tuple_as_f64(idx, &mut buf);
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

    fn make_field(vx: f64) -> ImageData {
        let dims = [10, 10, 10];
        let n = dims[0] * dims[1] * dims[2];
        let mut v = Vec::with_capacity(n * 3);
        for _ in 0..n {
            v.push(vx);
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
    fn temporal_trace() {
        let f1 = make_field(1.0);
        let f2 = make_field(2.0);
        let result = temporal_stream_trace(&[&f1, &f2], &[[2.0, 5.0, 5.0]], 0.1, 50);
        assert!(result.points.len() > 2);
        assert!(result.lines.num_cells() >= 1);
        assert!(result.point_data().get_array("IntegrationTime").is_some());
    }

    #[test]
    fn temporal_trace_respects_nonzero_extent() {
        let mut f1 = make_field(1.0);
        let mut f2 = make_field(1.0);
        f1.set_extent([10, 19, 0, 9, 0, 9]);
        f2.set_extent([10, 19, 0, 9, 0, 9]);

        let result = temporal_stream_trace(&[&f1, &f2], &[[12.0, 5.0, 5.0]], 0.1, 10);
        assert!(result.points.len() > 1);
        assert!(result.lines.num_cells() >= 1);
    }

    #[test]
    fn temporal_trace_flat_image() {
        let mut f1 = make_field(1.0);
        let mut f2 = make_field(1.0);
        f1.set_extent([0, 9, 0, 9, 0, 0]);
        f2.set_extent([0, 9, 0, 9, 0, 0]);

        let result = temporal_stream_trace(&[&f1, &f2], &[[2.0, 5.0, 0.0]], 0.1, 10);
        assert!(result.points.len() > 1);
        assert!(result.lines.num_cells() >= 1);
    }

    #[test]
    fn empty() {
        assert_eq!(
            temporal_stream_trace(&[], &[[0.0; 3]], 0.1, 10)
                .points
                .len(),
            0
        );
    }
}
