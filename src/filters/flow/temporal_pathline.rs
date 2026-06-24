//! Temporal path line filter and temporal statistics.
//!
//! - TemporalPathLineFilter: trace particle paths across time steps
//! - TemporalStatistics: compute min/max/mean/std of arrays across time steps

use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};

/// Trace particle paths across time steps.
///
/// For each point in the first time step, finds the nearest point in
/// subsequent time steps and connects them as a polyline.
/// Adds "Time" point data and "Speed" arrays.
pub fn temporal_pathlines(time_steps: &[PolyData], times: &[f64]) -> PolyData {
    if time_steps.is_empty() || times.len() != time_steps.len() {
        return PolyData::new();
    }

    let n_seeds = time_steps[0].points.len();
    let n_steps = time_steps.len();

    let mut points = crate::data::Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut time_data = Vec::new();
    let mut speed_data = Vec::new();

    for seed_idx in 0..n_seeds {
        let mut path_indices = Vec::with_capacity(n_steps);
        let mut prev_pos = time_steps[0].points.get(seed_idx);

        for (step_idx, step) in time_steps.iter().enumerate() {
            // Find nearest point in this time step
            let target = if step_idx == 0 {
                prev_pos
            } else {
                nearest_point(&step.points, prev_pos)
            };

            let pt_idx = points.len();
            points.push(target);
            path_indices.push(pt_idx as i64);
            time_data.push(times[step_idx]);

            if step_idx > 0 {
                let dx = target[0] - prev_pos[0];
                let dy = target[1] - prev_pos[1];
                let dz = target[2] - prev_pos[2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                let dt = times[step_idx] - times[step_idx - 1];
                speed_data.push(if dt.abs() > 1e-30 { dist / dt } else { 0.0 });
            } else {
                speed_data.push(0.0);
            }

            prev_pos = target;
        }

        if path_indices.len() >= 2 {
            lines.push_cell(&path_indices);
        }
    }

    let mut result = PolyData::new();
    result.points = points;
    result.lines = lines;
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

fn nearest_point(points: &crate::data::Points<f64>, target: [f64; 3]) -> [f64; 3] {
    let mut best = points.get(0);
    let mut best_dist = f64::MAX;
    for i in 0..points.len() {
        let p = points.get(i);
        let d =
            (p[0] - target[0]).powi(2) + (p[1] - target[1]).powi(2) + (p[2] - target[2]).powi(2);
        if d < best_dist {
            best_dist = d;
            best = p;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pathlines_basic() {
        let step0 = PolyData::from_triangles(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]], vec![]);
        let step1 = PolyData::from_triangles(vec![[0.1, 0.0, 0.0], [1.1, 0.0, 0.0]], vec![]);
        let result = temporal_pathlines(&[step0, step1], &[0.0, 1.0]);
        assert_eq!(result.lines.num_cells(), 2); // 2 seed points → 2 pathlines
        assert_eq!(result.points.len(), 4); // 2 seeds × 2 time steps
    }
}
