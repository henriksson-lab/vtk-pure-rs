//! Quadric error metric mesh simplification.

use crate::data::{CellArray, Points, PolyData};

/// Simplify mesh using quadric error metrics to target face count.
pub fn simplify_quadric(mesh: &PolyData, target_faces: usize) -> PolyData {
    let mut pts: Vec<[f64; 3]> = (0..mesh.points.len()).map(|i| mesh.points.get(i)).collect();
    let mut tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();

    let npts = pts.len();
    let mut remap: Vec<usize> = (0..npts).collect();
    let mut removed = vec![false; tris.len()];

    while tris.iter().filter(|_| true).count() - removed.iter().filter(|&&r| r).count()
        > target_faces
    {
        // Find cheapest edge to collapse
        let mut best_cost = f64::INFINITY;
        let mut best_edge = (0, 0);
        let mut best_tri = 0;

        for (ti, tri) in tris.iter().enumerate() {
            if removed[ti] {
                continue;
            }
            let v = [
                resolve(tri[0], &remap),
                resolve(tri[1], &remap),
                resolve(tri[2], &remap),
            ];
            for i in 0..3 {
                let a = v[i];
                let b = v[(i + 1) % 3];
                if a == b {
                    continue;
                }
                let cost = edge_collapse_cost(&pts, a, b);
                if cost < best_cost {
                    best_cost = cost;
                    best_edge = (a, b);
                    best_tri = ti;
                }
            }
        }

        if best_cost.is_infinite() {
            break;
        }
        let (a, b) = best_edge;
        // Collapse b into a (move a to midpoint)
        pts[a] = [
            (pts[a][0] + pts[b][0]) / 2.0,
            (pts[a][1] + pts[b][1]) / 2.0,
            (pts[a][2] + pts[b][2]) / 2.0,
        ];
        remap[b] = a;

        // Remove degenerate triangles
        for ti in 0..tris.len() {
            if removed[ti] {
                continue;
            }
            let v = [
                resolve(tris[ti][0], &remap),
                resolve(tris[ti][1], &remap),
                resolve(tris[ti][2], &remap),
            ];
            if v[0] == v[1] || v[1] == v[2] || v[2] == v[0] {
                removed[ti] = true;
            }
        }
    }

    // Build output
    let mut used = vec![false; npts];
    let remaining: Vec<[usize; 3]> = tris
        .iter()
        .enumerate()
        .filter(|(i, _)| !removed[*i])
        .map(|(_, t)| {
            let v = [
                resolve(t[0], &remap),
                resolve(t[1], &remap),
                resolve(t[2], &remap),
            ];
            for &vi in &v {
                used[vi] = true;
            }
            v
        })
        .collect();

    let mut pt_map = vec![0usize; npts];
    let mut new_pts = Points::<f64>::new();
    for i in 0..npts {
        if used[i] {
            pt_map[i] = new_pts.len();
            new_pts.push(pts[i]);
        }
    }
    let mut polys = CellArray::new();
    for t in &remaining {
        polys.push_cell(&[
            pt_map[t[0]] as i64,
            pt_map[t[1]] as i64,
            pt_map[t[2]] as i64,
        ]);
    }

    let mut result = PolyData::new();
    result.points = new_pts;
    result.polys = polys;
    result
}

fn resolve(mut v: usize, remap: &[usize]) -> usize {
    while remap[v] != v {
        v = remap[v];
    }
    v
}

fn edge_collapse_cost(pts: &[[f64; 3]], a: usize, b: usize) -> f64 {
    // Simple: edge length as cost
    let pa = pts[a];
    let pb = pts[b];
    (pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_simplify() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 4, 3], [1, 3, 2]],
        );
        let r = simplify_quadric(&mesh, 1);
        assert!(r.polys.num_cells() <= 2);
    }
    #[test]
    fn test_no_simplify() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = simplify_quadric(&mesh, 10);
        assert_eq!(r.polys.num_cells(), 1); // can't go above original
    }
}
