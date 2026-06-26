//! Simple uniform remeshing via iterative edge split/collapse.

use crate::data::{CellArray, Points, PolyData};

/// Uniform remesh to target edge length by splitting long edges and collapsing short ones.
pub fn remesh_to_edge_length(mesh: &PolyData, target_length: f64, iterations: usize) -> PolyData {
    if target_length <= 0.0 || iterations == 0 {
        return mesh.clone();
    }

    let mut pts: Vec<[f64; 3]> = (0..mesh.points.len()).map(|i| mesh.points.get(i)).collect();
    let mut tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();

    let high = target_length * 4.0 / 3.0;
    let low = target_length * 4.0 / 5.0;

    for _ in 0..iterations {
        // Split long edges
        let mut new_tris = Vec::new();
        let mut edge_mids: std::collections::HashMap<(usize, usize), usize> =
            std::collections::HashMap::new();
        for &[a, b, c] in &tris {
            let edges = [(a, b), (b, c), (c, a)];
            let lens: Vec<f64> = edges
                .iter()
                .map(|&(u, v)| edge_len(&pts[u], &pts[v]))
                .collect();
            let max_idx = if lens[0] >= lens[1] && lens[0] >= lens[2] {
                0
            } else if lens[1] >= lens[2] {
                1
            } else {
                2
            };
            if lens[max_idx] > high {
                let (u, v) = edges[max_idx];
                let mid = *edge_mids.entry((u.min(v), u.max(v))).or_insert_with(|| {
                    let m = [
                        (pts[u][0] + pts[v][0]) / 2.0,
                        (pts[u][1] + pts[v][1]) / 2.0,
                        (pts[u][2] + pts[v][2]) / 2.0,
                    ];
                    pts.push(m);
                    pts.len() - 1
                });
                let w = [a, b, c]
                    .iter()
                    .find(|&&x| x != u && x != v)
                    .copied()
                    .unwrap();
                new_tris.push([u, mid, w]);
                new_tris.push([mid, v, w]);
            } else {
                new_tris.push([a, b, c]);
            }
        }
        tris = new_tris;

        // Collapse short edges (simple: just merge to midpoint, skip if topology issue)
        let mut collapsed = vec![false; tris.len()];
        let mut remap: Vec<usize> = (0..pts.len()).collect();
        for ti in 0..tris.len() {
            if collapsed[ti] {
                continue;
            }
            let [a, b, c] = [
                resolve(tris[ti][0], &remap),
                resolve(tris[ti][1], &remap),
                resolve(tris[ti][2], &remap),
            ];
            let edges = [(a, b), (b, c), (c, a)];
            for &(u, v) in &edges {
                if u == v {
                    continue;
                }
                if edge_len(&pts[u], &pts[v]) < low {
                    let mid = [
                        (pts[u][0] + pts[v][0]) / 2.0,
                        (pts[u][1] + pts[v][1]) / 2.0,
                        (pts[u][2] + pts[v][2]) / 2.0,
                    ];
                    pts[u] = mid;
                    remap[v] = u;
                    break;
                }
            }
        }
        // Rebuild with remapped vertices, remove degenerate
        tris = tris
            .iter()
            .filter_map(|t| {
                let a = resolve(t[0], &remap);
                let b = resolve(t[1], &remap);
                let c = resolve(t[2], &remap);
                if a != b && b != c && c != a {
                    Some([a, b, c])
                } else {
                    None
                }
            })
            .collect();
    }

    // Build output
    let mut used = vec![false; pts.len()];
    for t in &tris {
        for &v in t {
            used[v] = true;
        }
    }
    let mut pt_map = vec![0usize; pts.len()];
    let mut new_pts = Points::<f64>::new();
    for i in 0..pts.len() {
        if used[i] {
            pt_map[i] = new_pts.len();
            new_pts.push(pts[i]);
        }
    }
    let mut polys = CellArray::new();
    for t in &tris {
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

fn edge_len(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn resolve(mut v: usize, remap: &[usize]) -> usize {
    while remap[v] != v {
        v = remap[v];
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_remesh() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = remesh_to_edge_length(&mesh, 3.0, 3);
        assert!(result.polys.num_cells() > 1); // should be subdivided
    }
    #[test]
    fn test_already_fine() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = remesh_to_edge_length(&mesh, 1.0, 2);
        assert!(result.polys.num_cells() >= 1);
    }
}
