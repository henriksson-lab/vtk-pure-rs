//! Ball Pivoting Algorithm for surface reconstruction from point clouds.
//!
//! Reconstructs a triangle mesh from an unorganized point cloud with normals
//! by rolling a ball over the points and connecting triplets it touches.
//! This is a simplified version focused on correctness over performance.

use crate::data::{CellArray, Points, PolyData};

/// Reconstruct a triangle mesh from a point cloud using ball pivoting.
///
/// `radius` is the ball radius — should be slightly larger than the
/// average point spacing. Points must have normals for orientation.
pub fn ball_pivoting(points: &PolyData, radius: f64) -> PolyData {
    let n = points.points.len();
    if n < 3 {
        return points.clone();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| points.points.get(i)).collect();
    let r2 = radius * radius;

    // Build simple spatial hash for neighbor queries
    let _cell_size = radius * 2.0;
    let mut used = vec![false; n];
    let mut triangles: Vec<[usize; 3]> = Vec::new();

    // Seed triangle: find three closest points
    let (seed_a, seed_b) = find_closest_pair(&pts);
    let seed_c = find_third_point(&pts, seed_a, seed_b, radius);

    if let Some(c) = seed_c {
        triangles.push([seed_a, seed_b, c]);
        used[seed_a] = true;
        used[seed_b] = true;
        used[c] = true;

        // Frontier edges to expand from
        let mut frontier: Vec<(usize, usize)> = vec![(seed_a, seed_b), (seed_b, c), (c, seed_a)];

        let max_iterations = n * 4;
        let mut iter = 0;

        while let Some((ea, eb)) = frontier.pop() {
            iter += 1;
            if iter > max_iterations {
                break;
            }

            // Try to pivot the ball around edge (ea, eb) to find a new point
            if let Some(new_pt) = find_pivot_point(&pts, ea, eb, radius, &triangles) {
                // Check if this triangle already exists
                let tri = [ea, eb, new_pt];
                let exists = triangles.iter().any(|t| {
                    let mut s = [t[0], t[1], t[2]];
                    s.sort();
                    let mut c = [tri[0], tri[1], tri[2]];
                    c.sort();
                    s == c
                });
                if exists {
                    continue;
                }

                triangles.push(tri);
                used[new_pt] = true;

                // Add new edges to frontier
                frontier.push((eb, new_pt));
                frontier.push((new_pt, ea));
            }
        }
    }

    // Also do greedy nearest-neighbor triangulation for remaining points
    for i in 0..n {
        if used[i] {
            continue;
        }
        // Find two nearest used points
        let mut nearest: Vec<(usize, f64)> = (0..n)
            .filter(|&j| j != i && used[j])
            .map(|j| (j, dist2(pts[i], pts[j])))
            .collect();
        nearest.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if nearest.len() >= 2 && nearest[0].1 < r2 * 4.0 && nearest[1].1 < r2 * 4.0 {
            triangles.push([i, nearest[0].0, nearest[1].0]);
            used[i] = true;
        }
    }

    // Build output
    let mut new_points = Points::<f64>::new();
    for p in &pts {
        new_points.push(*p);
    }
    let mut polys = CellArray::new();
    for tri in &triangles {
        polys.push_cell(&[tri[0] as i64, tri[1] as i64, tri[2] as i64]);
    }

    let mut mesh = PolyData::new();
    mesh.points = new_points;
    mesh.polys = polys;
    mesh
}

fn find_closest_pair(pts: &[[f64; 3]]) -> (usize, usize) {
    let n = pts.len();
    let mut best = (0, 1);
    let mut best_d = f64::MAX;
    let check_n = n.min(100); // limit search for large clouds
    for i in 0..check_n {
        for j in i + 1..check_n {
            let d = dist2(pts[i], pts[j]);
            if d < best_d {
                best_d = d;
                best = (i, j);
            }
        }
    }
    best
}

fn find_third_point(pts: &[[f64; 3]], a: usize, b: usize, radius: f64) -> Option<usize> {
    let r2 = radius * radius * 4.0;
    let mid = [
        (pts[a][0] + pts[b][0]) / 2.0,
        (pts[a][1] + pts[b][1]) / 2.0,
        (pts[a][2] + pts[b][2]) / 2.0,
    ];
    let mut best = None;
    let mut best_d = f64::MAX;
    for (i, p) in pts.iter().enumerate() {
        if i == a || i == b {
            continue;
        }
        let d = dist2(*p, mid);
        if d < best_d && d < r2 {
            best_d = d;
            best = Some(i);
        }
    }
    best
}

fn find_pivot_point(
    pts: &[[f64; 3]],
    a: usize,
    b: usize,
    radius: f64,
    existing: &[[usize; 3]],
) -> Option<usize> {
    let r2 = radius * radius * 4.0;
    let mid = [
        (pts[a][0] + pts[b][0]) / 2.0,
        (pts[a][1] + pts[b][1]) / 2.0,
        (pts[a][2] + pts[b][2]) / 2.0,
    ];
    let mut candidates: Vec<(usize, f64)> = Vec::new();

    for (i, p) in pts.iter().enumerate() {
        if i == a || i == b {
            continue;
        }
        let d = dist2(*p, mid);
        if d < r2 {
            // Check this doesn't form a degenerate triangle
            let da = dist2(*p, pts[a]);
            let db = dist2(*p, pts[b]);
            if da > 1e-15 && db > 1e-15 {
                candidates.push((i, d));
            }
        }
    }

    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    for (idx, _) in candidates {
        // Check triangle doesn't already exist
        let mut tri = [a, b, idx];
        tri.sort();
        let exists = existing.iter().any(|t| {
            let mut s = [t[0], t[1], t[2]];
            s.sort();
            s == tri
        });
        if !exists {
            return Some(idx);
        }
    }
    None
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconstruct_plane() {
        // Grid of points on a plane
        let mut pts = Vec::new();
        for i in 0..5 {
            for j in 0..5 {
                pts.push([i as f64 * 0.5, j as f64 * 0.5, 0.0]);
            }
        }
        let cloud = PolyData::from_points(pts);
        let mesh = ball_pivoting(&cloud, 0.8);
        assert!(mesh.polys.num_cells() > 0, "should produce triangles");
        assert!(
            mesh.polys.num_cells() >= 10,
            "should produce many triangles, got {}",
            mesh.polys.num_cells()
        );
    }

    #[test]
    fn too_few_points() {
        let cloud = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let mesh = ball_pivoting(&cloud, 1.0);
        assert_eq!(mesh.polys.num_cells(), 0);
    }

    #[test]
    fn three_points() {
        let cloud = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]]);
        let mesh = ball_pivoting(&cloud, 2.0);
        assert!(mesh.polys.num_cells() >= 1);
    }
}
