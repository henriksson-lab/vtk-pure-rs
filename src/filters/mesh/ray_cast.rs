use crate::data::PolyData;

/// Result of a ray-mesh intersection test.
#[derive(Debug, Clone)]
pub struct RayHit {
    /// Index of the hit cell in `vtkPolyData` cell order
    /// (verts, then lines, then polys, then strips).
    pub cell_id: usize,
    /// Hit point position.
    pub point: [f64; 3],
    /// Parametric distance along the ray.
    pub t: f64,
}

/// Cast a ray and find the first intersection with a triangle mesh.
///
/// Ray is defined by `origin` and `direction`. Returns the closest hit.
///
/// Delegates to [`crate::filters::mesh::ray_cast_mesh`], which holds the single
/// ray/triangle implementation.
pub fn ray_cast(input: &PolyData, origin: [f64; 3], direction: [f64; 3]) -> Option<RayHit> {
    crate::filters::mesh::ray_cast_mesh::ray_cast_first(input, origin, direction).map(convert_hit)
}

/// Cast a ray and find ALL intersections (sorted by distance).
///
/// Delegates to [`crate::filters::mesh::ray_cast_mesh::ray_cast_all`]; only the
/// hit record differs (this one carries just the cell id, point and `t`).
pub fn ray_cast_all(input: &PolyData, origin: [f64; 3], direction: [f64; 3]) -> Vec<RayHit> {
    crate::filters::mesh::ray_cast_mesh::ray_cast_all(input, origin, direction)
        .into_iter()
        .map(convert_hit)
        .collect()
}

fn convert_hit(hit: crate::filters::mesh::ray_cast_mesh::RayHit) -> RayHit {
    RayHit {
        cell_id: hit.cell_index,
        point: hit.point,
        t: hit.t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quad() -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd
    }

    #[test]
    fn hit_front() {
        let pd = make_quad();
        let hit = ray_cast(&pd, [0.5, 0.5, 1.0], [0.0, 0.0, -1.0]);
        assert!(hit.is_some());
        let h = hit.unwrap();
        assert!((h.point[2]).abs() < 1e-10);
        assert!((h.t - 1.0).abs() < 1e-10);
    }

    #[test]
    fn miss() {
        let pd = make_quad();
        let hit = ray_cast(&pd, [5.0, 5.0, 1.0], [0.0, 0.0, -1.0]);
        assert!(hit.is_none());
    }

    #[test]
    fn all_hits() {
        let pd = make_quad();
        let hits = ray_cast_all(&pd, [0.5, 0.5, 1.0], [0.0, 0.0, -1.0]);
        assert!(!hits.is_empty());
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        assert!(ray_cast(&pd, [0.0; 3], [0.0, 0.0, -1.0]).is_none());
    }

    #[test]
    fn polygon_fan_internal_edge_counts_as_one_cell_hit() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let hits = ray_cast_all(&pd, [0.5, 0.5, 1.0], [0.0, 0.0, -1.0]);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].cell_id, 0);
    }

    #[test]
    fn invalid_cell_ids_are_skipped() {
        let mut pd = make_quad();
        pd.polys.push_cell(&[-1, 0, 1]);
        pd.polys.push_cell(&[0, 1, 99]);

        let hits = ray_cast_all(&pd, [0.5, 0.5, 1.0], [0.0, 0.0, -1.0]);
        assert!(!hits.is_empty());
        assert!(hits.iter().all(|hit| hit.cell_id < 2));
    }
}
