use crate::data::{CellArray, Points, PolyData};

/// Clip a PolyData by a plane defined by a point and normal.
///
/// Keeps the half-space where `dot(p - origin, normal) >= 0`.
/// Triangles that cross the plane are split, generating new vertices on the plane.
pub fn clip_by_plane(input: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    let mut points = input.points.clone();
    let mut polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        // Classify each vertex
        let dists: Vec<f64> = cell
            .iter()
            .map(|&id| {
                let p = input.points.get(id as usize);
                (p[0] - origin[0]) * normal[0]
                    + (p[1] - origin[1]) * normal[1]
                    + (p[2] - origin[2]) * normal[2]
            })
            .collect();

        let all_inside = dists.iter().all(|&d| d >= 0.0);
        let all_outside = dists.iter().all(|&d| d < 0.0);

        if all_inside {
            polys.push_cell(cell);
        } else if all_outside {
            // Skip entirely
        } else {
            // Clip: generate new polygon from inside vertices and intersection points
            let clipped = clip_polygon(cell, &dists, &input.points, &mut points);
            if clipped.len() >= 3 {
                // Fan-triangulate the clipped polygon
                for i in 1..clipped.len() - 1 {
                    polys.push_cell(&[clipped[0], clipped[i], clipped[i + 1]]);
                }
            }
        }
    }

    // Compact: only keep referenced points
    let mut used = vec![false; points.len()];
    for ci in 0..polys.num_cells() {
        for &vid in polys.cell(ci) {
            used[vid as usize] = true;
        }
    }
    let mut point_map = vec![0i64; points.len()];
    let mut compact_points = Points::new();
    for i in 0..points.len() {
        if used[i] {
            point_map[i] = compact_points.len() as i64;
            compact_points.push(points.get(i));
        }
    }
    let mut compact_polys = CellArray::new();
    for ci in 0..polys.num_cells() {
        let cell = polys.cell(ci);
        let remapped: Vec<i64> = cell.iter().map(|&v| point_map[v as usize]).collect();
        compact_polys.push_cell(&remapped);
    }

    let mut output = PolyData::new();
    output.points = compact_points;
    output.polys = compact_polys;
    output
}

/// Clip a single polygon, returning new vertex indices for the clipped result.
fn clip_polygon(
    cell: &[i64],
    dists: &[f64],
    src_points: &Points<f64>,
    all_points: &mut Points<f64>,
) -> Vec<i64> {
    let n = cell.len();
    let mut result = Vec::new();

    for i in 0..n {
        let j = (i + 1) % n;
        let di = dists[i];
        let dj = dists[j];
        let vi = cell[i];
        let vj = cell[j];

        if di >= 0.0 {
            result.push(vi);
        }

        // If edge crosses the plane, add intersection point
        if (di >= 0.0) != (dj >= 0.0) {
            let t = di / (di - dj);
            let pi = src_points.get(vi as usize);
            let pj = src_points.get(vj as usize);
            let intersection = [
                pi[0] + t * (pj[0] - pi[0]),
                pi[1] + t * (pj[1] - pi[1]),
                pi[2] + t * (pj[2] - pi[2]),
            ];
            let new_id = all_points.len() as i64;
            all_points.push(intersection);
            result.push(new_id);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_triangle_keeps_inside() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        // Plane at origin with +Z normal (everything is on the plane)
        let result = clip_by_plane(&pd, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        // All points have z=0 which is on the plane (>= 0), so kept
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn clip_removes_outside() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, -1.0], [1.0, 0.0, -1.0], [0.0, 1.0, -1.0]],
            vec![[0, 1, 2]],
        );

        // Plane at origin, normal +Z → triangle is entirely below
        let result = clip_by_plane(&pd, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn clip_splits_triangle() {
        let pd = PolyData::from_triangles(
            vec![
                [-1.0, 0.0, 0.0], // inside (x < 0 → outside if normal is +X)
                [1.0, 0.0, 0.0],  // inside
                [1.0, 1.0, 0.0],  // inside
            ],
            vec![[0, 1, 2]],
        );

        // Clip by x=0 plane, keeping x >= 0
        let result = clip_by_plane(&pd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        // Point 0 is outside (x=-1), points 1,2 are inside
        // Should create a clipped polygon → triangulated
        assert!(result.polys.num_cells() >= 1);
        // All resulting points should have x >= -1e-6
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            if i >= 3 {
                // New intersection points should be on the plane
                assert!(p[0].abs() < 1e-10, "intersection point x={}", p[0]);
            }
        }
    }
}
