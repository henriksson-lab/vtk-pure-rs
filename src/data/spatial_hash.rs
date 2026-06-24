//! Spatial hash grid for fast neighbor queries on point clouds.

use std::collections::HashMap;

/// A spatial hash grid for O(1) average-case neighbor lookups.
///
/// Divides space into uniform cells and maps each point to a cell.
/// Much faster than KdTree for uniform point distributions.
pub struct SpatialHash {
    cell_size: f64,
    cells: HashMap<(i64, i64, i64), Vec<usize>>,
    points: Vec<[f64; 3]>,
}

impl SpatialHash {
    /// Build a spatial hash from a set of points.
    pub fn build(points: &[[f64; 3]], cell_size: f64) -> Self {
        let mut cells: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();
        for (i, p) in points.iter().enumerate() {
            let key = Self::cell_key(p, cell_size);
            cells.entry(key).or_default().push(i);
        }
        Self {
            cell_size,
            cells,
            points: points.to_vec(),
        }
    }

    /// Build from vtk-data Points.
    pub fn from_points(points: &crate::data::Points<f64>, cell_size: f64) -> Self {
        let pts: Vec<[f64; 3]> = points.to_vec();
        Self::build(&pts, cell_size)
    }

    /// Find all points within `radius` of `query`.
    pub fn find_within_radius(&self, query: [f64; 3], radius: f64) -> Vec<(usize, f64)> {
        let r2 = radius * radius;
        let cells_to_check = (radius / self.cell_size).ceil() as i64 + 1;
        let center = Self::cell_key(&query, self.cell_size);

        let mut results = Vec::new();
        for di in -cells_to_check..=cells_to_check {
            for dj in -cells_to_check..=cells_to_check {
                for dk in -cells_to_check..=cells_to_check {
                    let key = (center.0 + di, center.1 + dj, center.2 + dk);
                    if let Some(indices) = self.cells.get(&key) {
                        for &idx in indices {
                            let p = self.points[idx];
                            let dx = p[0] - query[0];
                            let dy = p[1] - query[1];
                            let dz = p[2] - query[2];
                            let d2 = dx * dx + dy * dy + dz * dz;
                            if d2 <= r2 {
                                results.push((idx, d2.sqrt()));
                            }
                        }
                    }
                }
            }
        }
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results
    }

    /// Number of occupied cells.
    pub fn num_cells(&self) -> usize {
        self.cells.len()
    }

    /// Number of points.
    pub fn num_points(&self) -> usize {
        self.points.len()
    }

    fn cell_key(p: &[f64; 3], cell_size: f64) -> (i64, i64, i64) {
        (
            (p[0] / cell_size).floor() as i64,
            (p[1] / cell_size).floor() as i64,
            (p[2] / cell_size).floor() as i64,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_query() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [5.0, 0.0, 0.0]];
        let hash = SpatialHash::build(&points, 2.0);
        let results = hash.find_within_radius([0.0, 0.0, 0.0], 1.5);
        assert_eq!(results.len(), 2); // points 0 and 1
    }

    #[test]
    fn no_results() {
        let points = vec![[10.0, 10.0, 10.0]];
        let hash = SpatialHash::build(&points, 1.0);
        let results = hash.find_within_radius([0.0, 0.0, 0.0], 1.0);
        assert!(results.is_empty());
    }

    #[test]
    fn sorted_by_distance() {
        let points = vec![[3.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let hash = SpatialHash::build(&points, 5.0);
        let results = hash.find_within_radius([0.0, 0.0, 0.0], 10.0);
        assert_eq!(results.len(), 3);
        assert!(results[0].1 <= results[1].1);
        assert!(results[1].1 <= results[2].1);
    }

    #[test]
    fn num_cells() {
        let points = vec![[0.0; 3], [10.0, 0.0, 0.0]];
        let hash = SpatialHash::build(&points, 1.0);
        assert_eq!(hash.num_cells(), 2);
        assert_eq!(hash.num_points(), 2);
    }
}
