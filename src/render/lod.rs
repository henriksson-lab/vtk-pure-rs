use crate::data::PolyData;

/// A level-of-detail entry with PolyData and a distance threshold.
#[derive(Debug, Clone)]
pub struct LodLevel {
    /// The mesh for this detail level.
    pub data: PolyData,
    /// Maximum camera distance at which this level is used.
    /// The highest-detail level should have the smallest distance.
    /// Use `f64::INFINITY` for the lowest-detail fallback.
    pub max_distance: f64,
}

/// Level-of-detail manager for an actor.
///
/// Stores multiple detail levels sorted by distance.
/// Call `select` with the camera distance to get the appropriate PolyData.
#[derive(Debug, Clone)]
pub struct LodSet {
    /// Levels sorted by max_distance ascending (highest detail first).
    levels: Vec<LodLevel>,
}

impl LodSet {
    /// Create a new LOD set from a list of levels.
    ///
    /// Levels are sorted by max_distance automatically.
    pub fn new(mut levels: Vec<LodLevel>) -> Self {
        levels.sort_by(|a, b| a.max_distance.partial_cmp(&b.max_distance).unwrap());
        Self { levels }
    }

    /// Create a LOD set with two levels: full detail and decimated.
    pub fn two_level(high: PolyData, low: PolyData, switch_distance: f64) -> Self {
        Self::new(vec![
            LodLevel {
                data: high,
                max_distance: switch_distance,
            },
            LodLevel {
                data: low,
                max_distance: f64::INFINITY,
            },
        ])
    }

    /// Select the appropriate detail level for the given camera distance.
    pub fn select(&self, distance: f64) -> &PolyData {
        for level in &self.levels {
            if distance <= level.max_distance {
                return &level.data;
            }
        }
        // Fallback to last (lowest detail)
        &self.levels.last().unwrap().data
    }

    /// Number of LOD levels.
    pub fn num_levels(&self) -> usize {
        self.levels.len()
    }

    /// Get all levels.
    pub fn levels(&self) -> &[LodLevel] {
        &self.levels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pd(n: usize) -> PolyData {
        let mut pts = vec![];
        for i in 0..n {
            pts.push([i as f64, 0.0, 0.0]);
        }
        let mut pd = PolyData::new();
        for p in pts {
            pd.points.push(p);
        }
        pd
    }

    #[test]
    fn two_level_lod() {
        let high = make_pd(100);
        let low = make_pd(10);
        let lod = LodSet::two_level(high, low, 5.0);

        assert_eq!(lod.num_levels(), 2);
        assert_eq!(lod.select(3.0).points.len(), 100); // close = high detail
        assert_eq!(lod.select(10.0).points.len(), 10); // far = low detail
    }

    #[test]
    fn multi_level_lod() {
        let lod = LodSet::new(vec![
            LodLevel {
                data: make_pd(1000),
                max_distance: 2.0,
            },
            LodLevel {
                data: make_pd(100),
                max_distance: 10.0,
            },
            LodLevel {
                data: make_pd(10),
                max_distance: f64::INFINITY,
            },
        ]);

        assert_eq!(lod.select(1.0).points.len(), 1000);
        assert_eq!(lod.select(5.0).points.len(), 100);
        assert_eq!(lod.select(50.0).points.len(), 10);
    }

    #[test]
    fn boundary_distance() {
        let lod = LodSet::two_level(make_pd(100), make_pd(10), 5.0);
        assert_eq!(lod.select(5.0).points.len(), 100); // exactly at threshold = high
        assert_eq!(lod.select(5.01).points.len(), 10);
    }
}
