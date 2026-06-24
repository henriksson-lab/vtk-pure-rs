//! In-memory cache for temporal datasets.
//!
//! Stores a fixed number of time steps and evicts oldest when full.

use std::collections::VecDeque;

use crate::data::PolyData;

/// Cache for temporal datasets with FIFO eviction.
#[derive(Debug, Clone)]
pub struct TemporalDataSetCache {
    entries: VecDeque<(f64, PolyData)>,
    capacity: usize,
}

impl TemporalDataSetCache {
    /// Create a cache with the given capacity (number of time steps to store).
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Insert a dataset at the given time. Evicts oldest if at capacity.
    pub fn insert(&mut self, time: f64, data: PolyData) {
        if self.capacity == 0 {
            return;
        }
        if let Some((_, cached)) = self.entries.iter_mut().find(|(t, _)| *t == time) {
            *cached = data;
            return;
        }
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back((time, data));
    }

    /// Get the dataset at exactly the given time, if cached.
    pub fn get(&self, time: f64) -> Option<&PolyData> {
        self.entries
            .iter()
            .find(|(t, _)| *t == time)
            .map(|(_, d)| d)
    }

    /// Get the dataset closest to the given time.
    pub fn get_nearest(&self, time: f64) -> Option<&PolyData> {
        self.entries
            .iter()
            .min_by(|(a, _), (b, _)| {
                (a - time)
                    .abs()
                    .partial_cmp(&(b - time).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(_, d)| d)
    }

    /// Get the two bracketing time steps for interpolation.
    pub fn bracket(&self, time: f64) -> Option<(&PolyData, &PolyData, f64)> {
        let mut before: Option<(f64, &PolyData)> = None;
        let mut after: Option<(f64, &PolyData)> = None;

        for (t, data) in &self.entries {
            if *t <= time && before.map_or(true, |(bt, _)| *t > bt) {
                before = Some((*t, data));
            }
            if *t >= time && after.map_or(true, |(at, _)| *t < at) {
                after = Some((*t, data));
            }
        }

        match (before, after) {
            (Some((t0, d0)), Some((t1, d1))) => {
                let alpha = if (t1 - t0).abs() > 1e-15 {
                    (time - t0) / (t1 - t0)
                } else {
                    0.0
                };
                Some((d0, d1, alpha))
            }
            _ => None,
        }
    }

    /// Number of cached time steps.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all cached entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// All cached times in order.
    pub fn times(&self) -> Vec<f64> {
        self.entries.iter().map(|(t, _)| *t).collect()
    }
}

/// Mesh topology cache — detects when geometry changes vs only data changes.
#[derive(Debug, Clone)]
pub struct MeshTopologyCache {
    last_num_points: usize,
    last_num_cells: usize,
    cached_data: Option<PolyData>,
}

impl MeshTopologyCache {
    pub fn new() -> Self {
        Self {
            last_num_points: 0,
            last_num_cells: 0,
            cached_data: None,
        }
    }

    /// Update cache. Returns true if topology changed (points/cells count differs).
    pub fn update(&mut self, data: &PolyData) -> bool {
        let np = data.points.len();
        let nc = data.polys.num_cells();
        let changed = np != self.last_num_points || nc != self.last_num_cells;
        self.last_num_points = np;
        self.last_num_cells = nc;
        self.cached_data = Some(data.clone());
        changed
    }

    /// Get cached data if available.
    pub fn get(&self) -> Option<&PolyData> {
        self.cached_data.as_ref()
    }

    /// Force static mesh: return cached topology with updated data arrays.
    pub fn force_static(&self, data: &PolyData) -> Option<PolyData> {
        let cached = self.cached_data.as_ref()?;
        let mut result = cached.clone();
        // Copy point data and cell data from input
        *result.point_data_mut() = data.point_data().clone();
        *result.cell_data_mut() = data.cell_data().clone();
        Some(result)
    }
}

impl Default for MeshTopologyCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pd(n: usize) -> PolyData {
        let pts: Vec<[f64; 3]> = (0..n).map(|i| [i as f64, 0.0, 0.0]).collect();
        PolyData::from_triangles(pts, vec![])
    }

    #[test]
    fn cache_insert_and_get() {
        let mut cache = TemporalDataSetCache::new(3);
        cache.insert(0.0, sample_pd(3));
        cache.insert(1.0, sample_pd(4));
        cache.insert(2.0, sample_pd(5));
        assert_eq!(cache.len(), 3);
        assert!(cache.get(1.0).is_some());
        assert_eq!(cache.get(1.0).unwrap().points.len(), 4);
    }

    #[test]
    fn cache_eviction() {
        let mut cache = TemporalDataSetCache::new(2);
        cache.insert(0.0, sample_pd(3));
        cache.insert(1.0, sample_pd(4));
        cache.insert(2.0, sample_pd(5));
        assert_eq!(cache.len(), 2);
        assert!(cache.get(0.0).is_none()); // evicted
        assert!(cache.get(1.0).is_some());
    }

    #[test]
    fn zero_capacity_disables_cache() {
        let mut cache = TemporalDataSetCache::new(0);
        cache.insert(0.0, sample_pd(3));
        assert_eq!(cache.len(), 0);
        assert!(cache.get(0.0).is_none());
    }

    #[test]
    fn insert_replaces_existing_time_without_growing() {
        let mut cache = TemporalDataSetCache::new(2);
        cache.insert(1.0, sample_pd(3));
        cache.insert(1.0, sample_pd(6));
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(1.0).unwrap().points.len(), 6);
    }

    #[test]
    fn bracket_interpolation() {
        let mut cache = TemporalDataSetCache::new(3);
        cache.insert(0.0, sample_pd(3));
        cache.insert(2.0, sample_pd(5));
        let (_, _, alpha) = cache.bracket(1.0).unwrap();
        assert!((alpha - 0.5).abs() < 1e-10);
    }

    #[test]
    fn bracket_uses_time_order_not_insertion_order() {
        let mut cache = TemporalDataSetCache::new(4);
        cache.insert(4.0, sample_pd(4));
        cache.insert(0.0, sample_pd(3));
        cache.insert(2.0, sample_pd(5));
        let (before, after, alpha) = cache.bracket(3.0).unwrap();
        assert_eq!(before.points.len(), 5);
        assert_eq!(after.points.len(), 4);
        assert!((alpha - 0.5).abs() < 1e-10);
    }

    #[test]
    fn mesh_topology_cache() {
        let mut mc = MeshTopologyCache::new();
        let pd1 = sample_pd(3);
        assert!(mc.update(&pd1)); // first update: "changed"
        assert!(!mc.update(&pd1)); // same topology: not changed

        let pd2 = sample_pd(5);
        assert!(mc.update(&pd2)); // different count: changed
    }
}
