use crate::data::{CellArray, Points, PolyData};

/// Compute a 2D Delaunay triangulation from a set of points.
///
/// Projects all points onto the XY plane for triangulation, then produces
/// a PolyData with triangle cells. Uses the Bowyer-Watson algorithm with
/// a super-triangle.
pub fn delaunay_2d(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n < 3 {
        return PolyData::new();
    }

    // Extract 2D coordinates
    let pts: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            let p = input.points.get(i);
            [p[0], p[1]]
        })
        .collect();

    // Compute bounding box
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    for p in &pts {
        min_x = min_x.min(p[0]);
        max_x = max_x.max(p[0]);
        min_y = min_y.min(p[1]);
        max_y = max_y.max(p[1]);
    }

    let dx = (max_x - min_x).max(1e-10);
    let dy = (max_y - min_y).max(1e-10);
    let margin = (dx + dy) * 10.0;

    // Super-triangle vertices (indices n, n+1, n+2)
    let super_pts = [
        [min_x - margin, min_y - margin],
        [max_x + margin * 2.0, min_y - margin],
        [min_x - margin, max_y + margin * 2.0],
    ];

    let mut all_pts: Vec<[f64; 2]> = pts.clone();
    all_pts.extend_from_slice(&super_pts);

    // Triangles: (a, b, c) indices into all_pts
    let mut triangles: Vec<[usize; 3]> = vec![[n, n + 1, n + 2]];

    // Bowyer-Watson: insert each point
    for pi in 0..n {
        let p = all_pts[pi];

        // Find triangles whose circumcircle contains the point
        let mut bad = vec![false; triangles.len()];
        for (ti, tri) in triangles.iter().enumerate() {
            if in_circumcircle(all_pts[tri[0]], all_pts[tri[1]], all_pts[tri[2]], p) {
                bad[ti] = true;
            }
        }

        // Find boundary polygon of the "hole"
        let mut polygon: Vec<(usize, usize)> = Vec::new();
        for (ti, tri) in triangles.iter().enumerate() {
            if !bad[ti] {
                continue;
            }
            let edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
            for &(a, b) in &edges {
                // Edge is on boundary if the adjacent triangle (sharing b,a) is not bad
                let shared = triangles.iter().enumerate().any(|(tj, other)| {
                    if tj == ti || !bad[tj] {
                        // We want: adjacent triangle is NOT bad
                        return false;
                    }
                    let oedges = [
                        (other[0], other[1]),
                        (other[1], other[2]),
                        (other[2], other[0]),
                    ];
                    oedges.contains(&(b, a))
                });
                if !shared {
                    polygon.push((a, b));
                }
            }
        }

        // Remove bad triangles
        let mut new_tris: Vec<[usize; 3]> = Vec::new();
        for (ti, tri) in triangles.iter().enumerate() {
            if !bad[ti] {
                new_tris.push(*tri);
            }
        }

        // Create new triangles from polygon edges to the new point
        for &(a, b) in &polygon {
            new_tris.push([a, b, pi]);
        }

        triangles = new_tris;
    }

    // Remove triangles that reference super-triangle vertices
    triangles.retain(|tri| tri[0] < n && tri[1] < n && tri[2] < n);

    // Build output
    let mut out_points = Points::<f64>::new();
    for i in 0..n {
        out_points.push(input.points.get(i));
    }

    let mut out_polys = CellArray::new();
    for tri in &triangles {
        out_polys.push_cell(&[tri[0] as i64, tri[1] as i64, tri[2] as i64]);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

fn in_circumcircle(a: [f64; 2], b: [f64; 2], c: [f64; 2], p: [f64; 2]) -> bool {
    let ax = a[0] - p[0];
    let ay = a[1] - p[1];
    let bx = b[0] - p[0];
    let by = b[1] - p[1];
    let cx = c[0] - p[0];
    let cy = c[1] - p[1];

    let det = ax * (by * (cx * cx + cy * cy) - cy * (bx * bx + by * by))
        - ay * (bx * (cx * cx + cy * cy) - cx * (bx * bx + by * by))
        + (ax * ax + ay * ay) * (bx * cy - by * cx);

    det > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangulate_square() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);

        let result = delaunay_2d(&pd);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.polys.num_cells(), 2); // 4 points -> 2 triangles
    }

    #[test]
    fn triangulate_grid() {
        let mut pd = PolyData::new();
        // 3x3 grid = 9 points
        for j in 0..3 {
            for i in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        let result = delaunay_2d(&pd);
        assert_eq!(result.points.len(), 9);
        // 9 points in a square grid -> 8 triangles
        assert_eq!(result.polys.num_cells(), 8);
    }

    #[test]
    fn triangulate_three_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);

        let result = delaunay_2d(&pd);
        assert_eq!(result.polys.num_cells(), 1);
    }
}
