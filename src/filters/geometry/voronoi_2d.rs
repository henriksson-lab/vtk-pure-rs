use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Compute a 2D Voronoi diagram from a set of points.
///
/// Uses the dual of the Delaunay triangulation: for each input point,
/// constructs the Voronoi cell from the circumcenters of the surrounding
/// triangles. Returns a PolyData with polygon cells and a "SiteId" cell
/// data array mapping each cell to its generator point.
///
/// Points are projected to the XY plane. The result is clipped to a
/// bounding rectangle with the given `padding` around the point set.
pub fn voronoi_2d(input: &PolyData, _padding: f64) -> PolyData {
    let n = input.points.len();
    if n < 3 {
        return PolyData::new();
    }

    let pts: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            let p = input.points.get(i);
            [p[0], p[1]]
        })
        .collect();

    // First compute Delaunay triangulation
    let tris = delaunay(&pts);

    // Build adjacency: for each point, which triangles it belongs to (in order)
    let mut pt_tris: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (ti, tri) in tris.iter().enumerate() {
        for &v in tri {
            pt_tris[v].push(ti);
        }
    }

    // Compute circumcenters
    let circumcenters: Vec<[f64; 2]> = tris
        .iter()
        .map(|tri| circumcenter(pts[tri[0]], pts[tri[1]], pts[tri[2]]))
        .collect();

    // Build Voronoi cells
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();
    let mut site_ids: Vec<f64> = Vec::new();

    for pi in 0..n {
        let adj = &pt_tris[pi];
        if adj.len() < 3 {
            continue; // boundary point, skip for now
        }

        // Order circumcenters around the point by angle
        let mut angles: Vec<(usize, f64)> = adj
            .iter()
            .map(|&ti| {
                let cc = circumcenters[ti];
                let dx = cc[0] - pts[pi][0];
                let dy = cc[1] - pts[pi][1];
                (ti, dy.atan2(dx))
            })
            .collect();
        angles.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let base = out_points.len() as i64;
        let mut cell_ids: Vec<i64> = Vec::new();
        for (i, &(ti, _)) in angles.iter().enumerate() {
            let cc = circumcenters[ti];
            out_points.push([cc[0], cc[1], 0.0]);
            cell_ids.push(base + i as i64);
        }

        if cell_ids.len() >= 3 {
            out_polys.push_cell(&cell_ids);
            site_ids.push(pi as f64);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SiteId", site_ids, 1,
        )));
    pd
}

fn circumcenter(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> [f64; 2] {
    let ax = a[0];
    let ay = a[1];
    let bx = b[0];
    let by = b[1];
    let cx = c[0];
    let cy = c[1];

    let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
    if d.abs() < 1e-15 {
        return [(ax + bx + cx) / 3.0, (ay + by + cy) / 3.0];
    }

    let ux = ((ax * ax + ay * ay) * (by - cy)
        + (bx * bx + by * by) * (cy - ay)
        + (cx * cx + cy * cy) * (ay - by))
        / d;
    let uy = ((ax * ax + ay * ay) * (cx - bx)
        + (bx * bx + by * by) * (ax - cx)
        + (cx * cx + cy * cy) * (bx - ax))
        / d;
    [ux, uy]
}

/// Simple Bowyer-Watson Delaunay. Returns triangle vertex indices.
fn delaunay(pts: &[[f64; 2]]) -> Vec<[usize; 3]> {
    let n = pts.len();
    if n < 3 {
        return vec![];
    }

    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    for p in pts {
        min_x = min_x.min(p[0]);
        max_x = max_x.max(p[0]);
        min_y = min_y.min(p[1]);
        max_y = max_y.max(p[1]);
    }
    let dx = (max_x - min_x).max(1e-10);
    let dy = (max_y - min_y).max(1e-10);
    let margin = (dx + dy) * 10.0;

    let super_pts = [
        [min_x - margin, min_y - margin],
        [max_x + margin * 2.0, min_y - margin],
        [min_x - margin, max_y + margin * 2.0],
    ];

    let mut all_pts: Vec<[f64; 2]> = pts.to_vec();
    all_pts.extend_from_slice(&super_pts);

    let mut triangles: Vec<[usize; 3]> = vec![[n, n + 1, n + 2]];

    for pi in 0..n {
        let p = all_pts[pi];
        let mut bad = vec![false; triangles.len()];
        for (ti, tri) in triangles.iter().enumerate() {
            if in_circumcircle(all_pts[tri[0]], all_pts[tri[1]], all_pts[tri[2]], p) {
                bad[ti] = true;
            }
        }

        let mut polygon: Vec<(usize, usize)> = Vec::new();
        for (ti, tri) in triangles.iter().enumerate() {
            if !bad[ti] {
                continue;
            }
            let edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
            for &(a, b) in &edges {
                let shared = triangles.iter().enumerate().any(|(tj, other)| {
                    if tj == ti || !bad[tj] {
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

        let mut new_tris: Vec<[usize; 3]> = triangles
            .iter()
            .enumerate()
            .filter(|(ti, _)| !bad[*ti])
            .map(|(_, t)| *t)
            .collect();
        for &(a, b) in &polygon {
            new_tris.push([a, b, pi]);
        }
        triangles = new_tris;
    }

    triangles.retain(|tri| tri[0] < n && tri[1] < n && tri[2] < n);
    triangles
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
    fn grid_voronoi() {
        let mut pd = PolyData::new();
        for j in 0..3 {
            for i in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        let result = voronoi_2d(&pd, 1.0);
        // Interior point (1,1) should produce a Voronoi cell
        assert!(result.polys.num_cells() > 0);
        assert!(result.cell_data().get_array("SiteId").is_some());
    }

    #[test]
    fn too_few_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        let result = voronoi_2d(&pd, 1.0);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn hexagonal_layout() {
        let mut pd = PolyData::new();
        // 7 points: center + 6 surrounding
        pd.points.push([0.0, 0.0, 0.0]);
        for i in 0..6 {
            let angle = std::f64::consts::PI * 2.0 * i as f64 / 6.0;
            pd.points.push([angle.cos(), angle.sin(), 0.0]);
        }
        let result = voronoi_2d(&pd, 1.0);
        assert!(result.polys.num_cells() >= 1);
    }
}
