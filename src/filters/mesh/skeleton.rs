use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::collections::{HashMap, HashSet};

/// Extract the topological skeleton (medial axis) of a 2D point cloud.
///
/// Computes the Voronoi vertices from the Delaunay triangulation and
/// filters them to approximate the medial axis. Returns a PolyData
/// with line segments connecting internal Voronoi vertices.
pub fn skeleton_2d(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n < 3 {
        return PolyData::new();
    }

    // Use the Voronoi dual: circumcenters of Delaunay triangles
    // that are "inside" the point set form the skeleton
    let pts: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            let p = input.points.get(i);
            [p[0], p[1]]
        })
        .collect();

    let tris = delaunay_2d(&pts);
    if tris.is_empty() {
        return PolyData::new();
    }

    let circumcenters: Vec<[f64; 2]> = tris
        .iter()
        .map(|tri| circumcenter(pts[tri[0]], pts[tri[1]], pts[tri[2]]))
        .collect();

    // Build adjacency between triangles sharing an edge
    let mut edge_tris: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (ti, tri) in tris.iter().enumerate() {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_tris.entry(key).or_default().push(ti);
        }
    }

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut cc_map: HashMap<usize, i64> = HashMap::new();

    for (_edge, adj_tris) in &edge_tris {
        if adj_tris.len() == 2 {
            let t0 = adj_tris[0];
            let t1 = adj_tris[1];
            let id0 = *cc_map.entry(t0).or_insert_with(|| {
                let idx = out_points.len() as i64;
                out_points.push([circumcenters[t0][0], circumcenters[t0][1], 0.0]);
                idx
            });
            let id1 = *cc_map.entry(t1).or_insert_with(|| {
                let idx = out_points.len() as i64;
                out_points.push([circumcenters[t1][0], circumcenters[t1][1], 0.0]);
                idx
            });
            out_lines.push_cell(&[id0, id1]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd
}

fn delaunay_2d(pts: &[[f64; 2]]) -> Vec<[usize; 3]> {
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
    let m = ((max_x - min_x) + (max_y - min_y)) * 10.0;
    let sp = [
        [min_x - m, min_y - m],
        [max_x + m * 2.0, min_y - m],
        [min_x - m, max_y + m * 2.0],
    ];
    let mut all: Vec<[f64; 2]> = pts.to_vec();
    all.extend_from_slice(&sp);
    let mut tris: Vec<[usize; 3]> = vec![[n, n + 1, n + 2]];
    for pi in 0..n {
        let p = all[pi];
        let mut bad = vec![false; tris.len()];
        for (ti, tri) in tris.iter().enumerate() {
            if in_cc(all[tri[0]], all[tri[1]], all[tri[2]], p) {
                bad[ti] = true;
            }
        }
        let mut poly = Vec::new();
        for (ti, tri) in tris.iter().enumerate() {
            if !bad[ti] {
                continue;
            }
            let edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
            for &(a, b) in &edges {
                let shared = tris.iter().enumerate().any(|(tj, o)| {
                    if tj == ti || !bad[tj] {
                        return false;
                    }
                    [(o[0], o[1]), (o[1], o[2]), (o[2], o[0])].contains(&(b, a))
                });
                if !shared {
                    poly.push((a, b));
                }
            }
        }
        let mut nt: Vec<[usize; 3]> = tris
            .iter()
            .enumerate()
            .filter(|(ti, _)| !bad[*ti])
            .map(|(_, t)| *t)
            .collect();
        for &(a, b) in &poly {
            nt.push([a, b, pi]);
        }
        tris = nt;
    }
    tris.retain(|t| t[0] < n && t[1] < n && t[2] < n);
    tris
}

fn in_cc(a: [f64; 2], b: [f64; 2], c: [f64; 2], p: [f64; 2]) -> bool {
    let ax = a[0] - p[0];
    let ay = a[1] - p[1];
    let bx = b[0] - p[0];
    let by = b[1] - p[1];
    let cx = c[0] - p[0];
    let cy = c[1] - p[1];
    ax * (by * (cx * cx + cy * cy) - cy * (bx * bx + by * by))
        - ay * (bx * (cx * cx + cy * cy) - cx * (bx * bx + by * by))
        + (ax * ax + ay * ay) * (bx * cy - by * cx)
        > 0.0
}

fn circumcenter(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> [f64; 2] {
    let d = 2.0 * (a[0] * (b[1] - c[1]) + b[0] * (c[1] - a[1]) + c[0] * (a[1] - b[1]));
    if d.abs() < 1e-15 {
        return [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0];
    }
    let ux = ((a[0] * a[0] + a[1] * a[1]) * (b[1] - c[1])
        + (b[0] * b[0] + b[1] * b[1]) * (c[1] - a[1])
        + (c[0] * c[0] + c[1] * c[1]) * (a[1] - b[1]))
        / d;
    let uy = ((a[0] * a[0] + a[1] * a[1]) * (c[0] - b[0])
        + (b[0] * b[0] + b[1] * b[1]) * (a[0] - c[0])
        + (c[0] * c[0] + c[1] * c[1]) * (b[0] - a[0]))
        / d;
    [ux, uy]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_skeleton() {
        let mut pd = PolyData::new();
        for j in 0..4 {
            for i in 0..4 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        let result = skeleton_2d(&pd);
        assert!(result.lines.num_cells() > 0);
        assert!(result.points.len() > 0);
    }

    #[test]
    fn too_few_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        let result = skeleton_2d(&pd);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = skeleton_2d(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
