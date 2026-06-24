//! Extract specific time steps from a TemporalDataSet.

use crate::data::{PolyData, TemporalDataSet};

/// Extract a single time step from a TemporalDataSet by index.
pub fn extract_time_step_by_index(temporal: &TemporalDataSet, index: usize) -> Option<PolyData> {
    temporal.step(index).cloned()
}

/// Extract a time step closest to the given time value.
pub fn extract_time_step_at_time(temporal: &TemporalDataSet, time: f64) -> Option<PolyData> {
    let times = temporal.times();
    if times.is_empty() {
        return None;
    }

    let mut best_idx = 0;
    let mut best_diff = (times[0] - time).abs();
    for (i, &t) in times.iter().enumerate().skip(1) {
        let diff = (t - time).abs();
        if diff < best_diff {
            best_diff = diff;
            best_idx = i;
        }
    }
    temporal.step(best_idx).cloned()
}

/// Extract a range of time steps by index range.
pub fn extract_time_step_range(
    temporal: &TemporalDataSet,
    start: usize,
    end: usize,
) -> TemporalDataSet {
    let mut result = TemporalDataSet::new();
    let times = temporal.times();
    for i in start..=end.min(times.len().saturating_sub(1)) {
        if let Some(mesh) = temporal.step(i) {
            result.add_step(times[i], mesh.clone());
        }
    }
    result
}

/// Extract every Nth time step (subsampling).
pub fn extract_every_nth_step(temporal: &TemporalDataSet, n: usize) -> TemporalDataSet {
    let mut result = TemporalDataSet::new();
    let times = temporal.times();
    for (i, &t) in times.iter().enumerate() {
        if i % n.max(1) == 0 {
            if let Some(mesh) = temporal.step(i) {
                result.add_step(t, mesh.clone());
            }
        }
    }
    result
}

/// Snap a time value to the nearest available time step.
pub fn snap_to_nearest_time_step(temporal: &TemporalDataSet, time: f64) -> Option<f64> {
    let times = temporal.times();
    if times.is_empty() {
        return None;
    }
    let mut best = times[0];
    let mut best_diff = (best - time).abs();
    for &t in &times[1..] {
        let diff = (t - time).abs();
        if diff < best_diff {
            best_diff = diff;
            best = t;
        }
    }
    Some(best)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temporal() -> TemporalDataSet {
        let mut ts = TemporalDataSet::new();
        for i in 0..5 {
            let mesh = PolyData::from_points(vec![[i as f64, 0.0, 0.0]]);
            ts.add_step(i as f64 * 0.5, mesh);
        }
        ts
    }

    #[test]
    fn by_index() {
        let ts = make_temporal();
        let mesh = extract_time_step_by_index(&ts, 2).unwrap();
        let p = mesh.points.get(0);
        assert!((p[0] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn at_time() {
        let ts = make_temporal();
        let mesh = extract_time_step_at_time(&ts, 0.7).unwrap();
        // Closest to 0.7 is t=0.5 (index 1) or t=1.0 (index 2)
        let p = mesh.points.get(0);
        assert!(p[0] >= 1.0 && p[0] <= 2.0);
    }

    #[test]
    fn range() {
        let ts = make_temporal();
        let sub = extract_time_step_range(&ts, 1, 3);
        assert_eq!(sub.times().len(), 3);
    }

    #[test]
    fn every_nth() {
        let ts = make_temporal();
        let sub = extract_every_nth_step(&ts, 2);
        assert_eq!(sub.times().len(), 3); // indices 0, 2, 4
    }

    #[test]
    fn snap() {
        let ts = make_temporal();
        let t = snap_to_nearest_time_step(&ts, 0.6).unwrap();
        assert!((t - 0.5).abs() < 1e-10);
    }
}
