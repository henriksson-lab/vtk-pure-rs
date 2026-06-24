use crate::data::{CellArray, Points, PolyData};

/// 2D Delaunay triangulation using the Bowyer-Watson algorithm.
///
/// Takes a set of 2D points and produces a triangulated PolyData.
/// Points are projected onto z=0.
pub fn delaunay_2d_from_points(points: &[[f64; 2]]) -> PolyData {
    if points.len() < 3 {
        let mut pd = PolyData::new();
        for p in points {
            pd.points.push([p[0], p[1], 0.0]);
        }
        return pd;
    }

    // Find bounding box
    let mut min_x: f64 = f64::MAX;
    let mut min_y: f64 = f64::MAX;
    let mut max_x: f64 = f64::MIN;
    let mut max_y: f64 = f64::MIN;
    for p in points {
        if p[0] < min_x {
            min_x = p[0];
        }
        if p[1] < min_y {
            min_y = p[1];
        }
        if p[0] > max_x {
            max_x = p[0];
        }
        if p[1] > max_y {
            max_y = p[1];
        }
    }

    let dx: f64 = max_x - min_x;
    let dy: f64 = max_y - min_y;
    let delta: f64 = dx.max(dy).max(1e-10);

    // Create super-triangle that contains all points
    // Use a large triangle well outside the bounding box
    let margin: f64 = delta * 10.0;
    let cx: f64 = (min_x + max_x) * 0.5;
    let cy: f64 = (min_y + max_y) * 0.5;

    let st0: [f64; 2] = [cx - margin * 2.0, cy - margin];
    let st1: [f64; 2] = [cx + margin * 2.0, cy - margin];
    let st2: [f64; 2] = [cx, cy + margin * 2.0];

    // All vertices: super-triangle first, then input points
    let n_input: usize = points.len();
    let mut all_pts: Vec<[f64; 2]> = Vec::with_capacity(n_input + 3);
    all_pts.push(st0);
    all_pts.push(st1);
    all_pts.push(st2);
    for p in points {
        all_pts.push(*p);
    }

    // Triangle represented as 3 vertex indices
    let mut triangles: Vec<[usize; 3]> = vec![[0, 1, 2]];

    // Insert points one at a time
    for pi in 3..all_pts.len() {
        let px: f64 = all_pts[pi][0];
        let py: f64 = all_pts[pi][1];

        // Find triangles whose circumcircle contains the point
        let mut bad_triangles: Vec<usize> = Vec::new();
        for (ti, tri) in triangles.iter().enumerate() {
            if in_circumcircle(&all_pts, tri, px, py) {
                bad_triangles.push(ti);
            }
        }

        // Find boundary polygon of the hole
        let mut polygon: Vec<[usize; 2]> = Vec::new();
        for &ti in &bad_triangles {
            let tri = triangles[ti];
            let edges: [[usize; 2]; 3] = [[tri[0], tri[1]], [tri[1], tri[2]], [tri[2], tri[0]]];
            for edge in &edges {
                let shared: bool = bad_triangles
                    .iter()
                    .any(|&oi| oi != ti && triangle_has_edge(&triangles[oi], edge));
                if !shared {
                    polygon.push(*edge);
                }
            }
        }

        // Remove bad triangles (in reverse order to preserve indices)
        bad_triangles.sort_unstable();
        for &ti in bad_triangles.iter().rev() {
            triangles.swap_remove(ti);
        }

        // Create new triangles from polygon edges to the new point
        for edge in &polygon {
            triangles.push([edge[0], edge[1], pi]);
        }
    }

    // Remove triangles that share vertices with the super-triangle
    triangles.retain(|tri| tri[0] >= 3 && tri[1] >= 3 && tri[2] >= 3);

    // Build output PolyData (remap indices: subtract 3 for super-triangle offset)
    let mut out_points = Points::<f64>::new();
    for p in points {
        out_points.push([p[0], p[1], 0.0]);
    }

    let mut out_polys = CellArray::new();
    for tri in &triangles {
        let i0: i64 = (tri[0] - 3) as i64;
        let i1: i64 = (tri[1] - 3) as i64;
        let i2: i64 = (tri[2] - 3) as i64;
        out_polys.push_cell(&[i0, i1, i2]);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

/// Test whether point (px, py) lies inside the circumcircle of a triangle.
fn in_circumcircle(pts: &[[f64; 2]], tri: &[usize; 3], px: f64, py: f64) -> bool {
    let ax: f64 = pts[tri[0]][0];
    let ay: f64 = pts[tri[0]][1];
    let bx: f64 = pts[tri[1]][0];
    let by: f64 = pts[tri[1]][1];
    let cx: f64 = pts[tri[2]][0];
    let cy: f64 = pts[tri[2]][1];

    let dax: f64 = ax - px;
    let day: f64 = ay - py;
    let dbx: f64 = bx - px;
    let dby: f64 = by - py;
    let dcx: f64 = cx - px;
    let dcy: f64 = cy - py;

    let det: f64 = dax * (dby * (dcx * dcx + dcy * dcy) - dcy * (dbx * dbx + dby * dby))
        - day * (dbx * (dcx * dcx + dcy * dcy) - dcx * (dbx * dbx + dby * dby))
        + (dax * dax + day * day) * (dbx * dcy - dby * dcx);

    // For counter-clockwise triangles, det > 0 means inside circumcircle.
    // We handle both orientations by checking abs.
    // Actually, the sign depends on orientation; use the sign-aware check:
    // If the triangle is CCW, det > 0 means inside.
    // If CW, det < 0 means inside.
    // We check orientation first.
    let orient: f64 = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);
    if orient > 0.0 {
        det > 0.0
    } else {
        det < 0.0
    }
}

/// Check if a triangle contains a given edge (unordered).
fn triangle_has_edge(tri: &[usize; 3], edge: &[usize; 2]) -> bool {
    let edges: [[usize; 2]; 3] = [[tri[0], tri[1]], [tri[1], tri[2]], [tri[2], tri[0]]];
    for e in &edges {
        if (e[0] == edge[0] && e[1] == edge[1]) || (e[0] == edge[1] && e[1] == edge[0]) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_from_three_points() {
        let pts: Vec<[f64; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let result = delaunay_2d_from_points(&pts);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn square_produces_two_triangles() {
        let pts: Vec<[f64; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let result = delaunay_2d_from_points(&pts);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn fewer_than_three_points() {
        let pts: Vec<[f64; 2]> = vec![[0.0, 0.0], [1.0, 0.0]];
        let result = delaunay_2d_from_points(&pts);
        assert_eq!(result.points.len(), 2);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
