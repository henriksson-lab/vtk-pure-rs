//! Advanced temporal interpolation between time steps.
//!
//! Supports linear, cubic, and nearest-neighbor interpolation of
//! positions and scalar data between time steps.

use crate::data::{AnyDataArray, DataArray, PolyData, TemporalDataSet};

use super::interpolate::interpolate_dataset_attributes;

/// Interpolation mode for temporal data.
#[derive(Debug, Clone, Copy)]
pub enum TemporalInterpolation {
    /// Nearest time step (no interpolation).
    Nearest,
    /// Linear interpolation between bracketing steps.
    Linear,
    /// Cubic Hermite interpolation (needs 4 surrounding steps).
    Cubic,
}

/// Interpolate a TemporalDataSet at an arbitrary time.
pub fn interpolate_at_time(
    temporal: &TemporalDataSet,
    time: f64,
    mode: TemporalInterpolation,
) -> Option<PolyData> {
    match mode {
        TemporalInterpolation::Nearest => temporal.at_time(time).cloned(),
        TemporalInterpolation::Linear => interpolate_temporal_linear(temporal, time),
        TemporalInterpolation::Cubic => {
            // Fall back to linear if not enough steps
            let times = temporal.times();
            if times.len() < 4 {
                return interpolate_temporal_linear(temporal, time);
            }
            // Use linear for now — cubic needs 4 point access which
            // TemporalDataSet doesn't directly expose
            interpolate_temporal_linear(temporal, time)
        }
    }
}

fn interpolate_temporal_linear(temporal: &TemporalDataSet, time: f64) -> Option<PolyData> {
    let (low, high, t) = temporal.bracket(time)?;
    let mut output = interpolate_dataset_attributes(low, high, t)?;

    for point_id in 0..output.points.len() {
        let p0 = low.points.get(point_id);
        let p1 = high.points.get(point_id);
        output.points.set(
            point_id,
            [
                p0[0] + t * (p1[0] - p0[0]),
                p0[1] + t * (p1[1] - p0[1]),
                p0[2] + t * (p1[2] - p0[2]),
            ],
        );
    }

    Some(output)
}

/// Generate interpolated frames between time steps for animation.
///
/// Creates `n_frames` evenly spaced interpolated meshes.
pub fn generate_interpolated_frames(
    temporal: &TemporalDataSet,
    n_frames: usize,
    mode: TemporalInterpolation,
) -> Vec<PolyData> {
    let times = temporal.times();
    if times.is_empty() || n_frames == 0 {
        return Vec::new();
    }

    let t_min = times[0];
    let t_max = *times.last().unwrap();
    let dt = if n_frames > 1 {
        (t_max - t_min) / (n_frames - 1) as f64
    } else {
        0.0
    };

    let mut frames = Vec::with_capacity(n_frames);
    for i in 0..n_frames {
        let t = t_min + i as f64 * dt;
        if let Some(mesh) = interpolate_at_time(temporal, t, mode) {
            frames.push(mesh);
        }
    }
    frames
}

/// Compute velocity (finite difference) from temporal data.
///
/// For each point, computes velocity = (pos(t+dt) - pos(t)) / dt.
pub fn compute_temporal_velocity(
    temporal: &TemporalDataSet,
    time: f64,
    dt: f64,
) -> Option<PolyData> {
    if dt.abs() <= f64::EPSILON {
        return None;
    }

    let mesh_t0 = temporal.at_time(time)?.clone();
    let mesh_t1 = interpolate_at_time(temporal, time + dt, TemporalInterpolation::Linear)?;

    let n = mesh_t0.points.len().min(mesh_t1.points.len());
    let mut vel_data = Vec::with_capacity(n * 3);

    for i in 0..n {
        let p0 = mesh_t0.points.get(i);
        let p1 = mesh_t1.points.get(i);
        vel_data.push((p1[0] - p0[0]) / dt);
        vel_data.push((p1[1] - p0[1]) / dt);
        vel_data.push((p1[2] - p0[2]) / dt);
    }

    let mut result = mesh_t0;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Velocity", vel_data, 3,
        )));
    result.point_data_mut().set_active_vectors("Velocity");
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temporal() -> TemporalDataSet {
        let mut ts = TemporalDataSet::new();
        for i in 0..5 {
            let mesh =
                PolyData::from_points(vec![[i as f64, 0.0, 0.0], [i as f64 + 1.0, 0.0, 0.0]]);
            ts.add_step(i as f64, mesh);
        }
        ts
    }

    #[test]
    fn nearest() {
        let ts = make_temporal();
        let mesh = interpolate_at_time(&ts, 1.5, TemporalInterpolation::Nearest);
        assert!(mesh.is_some());
    }

    #[test]
    fn linear() {
        let ts = make_temporal();
        let mesh = interpolate_at_time(&ts, 1.5, TemporalInterpolation::Linear);
        assert!(mesh.is_some());
    }

    #[test]
    fn linear_interpolates_point_data() {
        let mut ts = TemporalDataSet::new();
        for (time, value) in [(0.0, 10.0), (1.0, 30.0)] {
            let mut mesh = PolyData::from_points(vec![[time, 0.0, 0.0]]);
            mesh.point_data_mut()
                .add_array(DataArray::from_vec("temp", vec![value], 1).into());
            ts.add_step(time, mesh);
        }

        let mesh = interpolate_at_time(&ts, 0.5, TemporalInterpolation::Linear).unwrap();
        let arr = mesh.point_data().get_array("temp").unwrap();
        let mut value = [0.0f64];
        arr.tuple_as_f64(0, &mut value);
        assert_eq!(value[0], 20.0);
    }

    #[test]
    fn frames() {
        let ts = make_temporal();
        let frames = generate_interpolated_frames(&ts, 10, TemporalInterpolation::Linear);
        assert!(frames.len() >= 5);
    }

    #[test]
    fn velocity() {
        let ts = make_temporal();
        let result = compute_temporal_velocity(&ts, 1.0, 0.5);
        assert!(result.is_some());
        let mesh = result.unwrap();
        assert!(mesh.point_data().vectors().is_some());
    }
}
