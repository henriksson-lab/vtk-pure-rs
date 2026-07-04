use crate::data::{CellLocator, PolyData};

/// Result of computing distance statistics between two meshes.
#[derive(Debug, Clone, Copy)]
pub struct MeshDistanceResult {
    /// Maximum distance from any point in A to its closest point in B.
    pub max_a_to_b: f64,
    /// Maximum distance from any point in B to its closest point in A.
    pub max_b_to_a: f64,
    /// Mean distance from points in A to their closest points in B.
    pub mean_a_to_b: f64,
    /// Mean distance from points in B to their closest points in A.
    pub mean_b_to_a: f64,
}

/// Compute symmetric Hausdorff-like distance statistics between two meshes.
///
/// For each point in mesh A, finds the closest point in mesh B (brute force)
/// and vice versa. Returns max and mean distances in both directions.
pub fn mesh_distance_stats(a: &PolyData, b: &PolyData) -> MeshDistanceResult {
    let (max_ab, mean_ab) = directed_distance(a, b);
    let (max_ba, mean_ba) = directed_distance(b, a);
    MeshDistanceResult {
        max_a_to_b: max_ab,
        max_b_to_a: max_ba,
        mean_a_to_b: mean_ab,
        mean_b_to_a: mean_ba,
    }
}

/// Compute directed distance stats from A to B.
/// Returns (max_distance, mean_distance).
fn directed_distance(a: &PolyData, b: &PolyData) -> (f64, f64) {
    let na: usize = a.points.len();
    if na == 0 {
        return (0.0, 0.0);
    }
    let locator = CellLocator::build(b);
    let use_locator = locator.num_primitives() > 0;
    if !use_locator && b.points.len() == 0 {
        return (0.0, 0.0);
    }

    let mut max_d: f64 = 0.0;
    let mut sum_d: f64 = 0.0;

    for i in 0..na {
        let pa = a.points.get(i);
        let d: f64 = if use_locator {
            locator
                .find_closest_cell(pa)
                .map(|(_, _, d2)| d2.sqrt())
                .unwrap_or(0.0)
        } else {
            min_distance_to_points(pa, b)
        };
        if d > max_d {
            max_d = d;
        }
        sum_d += d;
    }

    (max_d, sum_d / na as f64)
}

fn min_distance_to_points(p: [f64; 3], mesh: &PolyData) -> f64 {
    let mut best = f64::INFINITY;
    for i in 0..mesh.points.len() {
        let q = mesh.points.get(i);
        let dx = p[0] - q[0];
        let dy = p[1] - q[1];
        let dz = p[2] - q[2];
        best = best.min((dx * dx + dy * dy + dz * dz).sqrt());
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_meshes() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);
        a.points.push([0.0, 1.0, 0.0]);

        let result = mesh_distance_stats(&a, &a);
        assert!(result.max_a_to_b < 1e-10);
        assert!(result.max_b_to_a < 1e-10);
        assert!(result.mean_a_to_b < 1e-10);
        assert!(result.mean_b_to_a < 1e-10);
    }

    #[test]
    fn known_distance() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);

        let mut b = PolyData::new();
        b.points.push([3.0, 4.0, 0.0]);

        let result = mesh_distance_stats(&a, &b);
        assert!((result.max_a_to_b - 5.0).abs() < 1e-10);
        assert!((result.max_b_to_a - 5.0).abs() < 1e-10);
        assert!((result.mean_a_to_b - 5.0).abs() < 1e-10);
    }

    #[test]
    fn asymmetric_distances() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);

        let mut b = PolyData::new();
        b.points.push([0.0, 0.0, 0.0]);
        b.points.push([10.0, 0.0, 0.0]);

        let result = mesh_distance_stats(&a, &b);
        // A->B: point (0,0,0) closest to (0,0,0) = 0
        assert!(result.max_a_to_b < 1e-10);
        // B->A: point (10,0,0) closest to (0,0,0) = 10
        assert!((result.max_b_to_a - 10.0).abs() < 1e-10);
        // mean B->A: (0 + 10) / 2 = 5
        assert!((result.mean_b_to_a - 5.0).abs() < 1e-10);
    }

    #[test]
    fn uses_surface_not_only_vertices() {
        let a = PolyData::from_points(vec![[0.25, 0.25, 1.0]]);
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = mesh_distance_stats(&a, &b);
        assert!((result.max_a_to_b - 1.0).abs() < 1e-10);
    }
}
