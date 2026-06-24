use crate::data::PolyData;

/// A temporal (time-varying) dataset containing snapshots at different time steps.
///
/// Each time step holds a PolyData and a time value. Supports interpolation
/// between time steps for animation.
#[derive(Debug, Clone)]
pub struct TemporalDataSet {
    steps: Vec<(f64, PolyData)>,
}

impl TemporalDataSet {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Add a time step with a PolyData snapshot.
    pub fn add_step(&mut self, time: f64, data: PolyData) {
        self.steps.push((time, data));
        self.steps.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    /// Number of time steps.
    pub fn num_steps(&self) -> usize {
        self.steps.len()
    }

    /// Time range [min, max].
    pub fn time_range(&self) -> Option<[f64; 2]> {
        if self.steps.is_empty() {
            return None;
        }
        Some([self.steps.first().unwrap().0, self.steps.last().unwrap().0])
    }

    /// Get all time values.
    pub fn times(&self) -> Vec<f64> {
        self.steps.iter().map(|(t, _)| *t).collect()
    }

    /// Get the PolyData at a specific time step index.
    pub fn step(&self, idx: usize) -> Option<&PolyData> {
        self.steps.get(idx).map(|(_, pd)| pd)
    }

    /// Get the PolyData nearest to the given time.
    pub fn at_time(&self, time: f64) -> Option<&PolyData> {
        if self.steps.is_empty() {
            return None;
        }
        let idx = self
            .steps
            .iter()
            .enumerate()
            .min_by(|(_, (ta, _)), (_, (tb, _))| {
                (ta - time).abs().partial_cmp(&(tb - time).abs()).unwrap()
            })
            .map(|(i, _)| i)?;
        Some(&self.steps[idx].1)
    }

    /// Find the two bracketing steps for interpolation at a given time.
    /// Returns (step_a, step_b, interpolation_factor).
    pub fn bracket(&self, time: f64) -> Option<(&PolyData, &PolyData, f64)> {
        if self.steps.len() < 2 {
            return None;
        }
        for i in 0..self.steps.len() - 1 {
            let (t0, pd0) = &self.steps[i];
            let (t1, pd1) = &self.steps[i + 1];
            if time >= *t0 && time <= *t1 {
                let frac = if (t1 - t0).abs() > 1e-15 {
                    (time - t0) / (t1 - t0)
                } else {
                    0.0
                };
                return Some((pd0, pd1, frac));
            }
        }
        None
    }

    /// Interpolate point positions between two bracketing time steps.
    pub fn interpolate_positions(&self, time: f64) -> Option<PolyData> {
        let (pd0, pd1, t) = self.bracket(time)?;
        if pd0.points.len() != pd1.points.len() {
            return None;
        }

        let mut result = pd0.clone();
        for i in 0..result.points.len() {
            let p0 = pd0.points.get(i);
            let p1 = pd1.points.get(i);
            result.points.set(
                i,
                [
                    p0[0] + t * (p1[0] - p0[0]),
                    p0[1] + t * (p1[1] - p0[1]),
                    p0[2] + t * (p1[2] - p0[2]),
                ],
            );
        }
        Some(result)
    }
}

impl Default for TemporalDataSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pd(x_offset: f64) -> PolyData {
        PolyData::from_triangles(
            vec![
                [x_offset, 0.0, 0.0],
                [x_offset + 1.0, 0.0, 0.0],
                [x_offset, 1.0, 0.0],
            ],
            vec![[0, 1, 2]],
        )
    }

    #[test]
    fn basic_temporal() {
        let mut ts = TemporalDataSet::new();
        ts.add_step(0.0, make_pd(0.0));
        ts.add_step(1.0, make_pd(5.0));
        assert_eq!(ts.num_steps(), 2);
        assert_eq!(ts.time_range(), Some([0.0, 1.0]));
    }

    #[test]
    fn at_time_nearest() {
        let mut ts = TemporalDataSet::new();
        ts.add_step(0.0, make_pd(0.0));
        ts.add_step(1.0, make_pd(5.0));
        let pd = ts.at_time(0.3).unwrap();
        assert!((pd.points.get(0)[0]).abs() < 1e-10); // nearer to t=0
    }

    #[test]
    fn bracket() {
        let mut ts = TemporalDataSet::new();
        ts.add_step(0.0, make_pd(0.0));
        ts.add_step(1.0, make_pd(10.0));
        let (_, _, frac) = ts.bracket(0.5).unwrap();
        assert!((frac - 0.5).abs() < 1e-10);
    }

    #[test]
    fn interpolate() {
        let mut ts = TemporalDataSet::new();
        ts.add_step(0.0, make_pd(0.0));
        ts.add_step(1.0, make_pd(10.0));
        let interp = ts.interpolate_positions(0.5).unwrap();
        assert!((interp.points.get(0)[0] - 5.0).abs() < 1e-10);
    }
}
