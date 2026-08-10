//! Compute per-face areas and attach as cell data.
//!
//! The implementations live in [`crate::filters::mesh::face_area_stats`] (per-face
//! areas and their statistics) and [`crate::filters::mesh::surface_curvature_integral`]
//! (total area); this module only re-exports them so there is a single
//! implementation of each.

use crate::data::PolyData;

pub use crate::filters::mesh::face_area_stats::compute_face_areas;
pub use crate::filters::mesh::surface_curvature_integral::total_surface_area;

/// Compute min, max, average face area.
///
/// Thin wrapper over [`crate::filters::mesh::face_area_stats::face_area_stats`],
/// which returns the richer [`FaceAreaStats`](crate::filters::mesh::face_area_stats::FaceAreaStats)
/// record. An empty mesh yields `(0, 0, 0)`.
pub fn face_area_stats(mesh: &PolyData) -> (f64, f64, f64) {
    match crate::filters::mesh::face_area_stats::face_area_stats(mesh) {
        Some(stats) => (stats.min_area, stats.max_area, stats.mean_area),
        None => (0.0, 0.0, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_total() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let total = total_surface_area(&mesh);
        assert!((total - 1.0).abs() < 1e-10); // unit square = 2 * 0.5
    }
    #[test]
    fn test_stats() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 2.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let (mn, mx, avg) = face_area_stats(&mesh);
        assert!((mn - 0.5).abs() < 1e-10);
        assert!((mx - 2.0).abs() < 1e-10);
        assert!((avg - 1.25).abs() < 1e-10);
    }
}
