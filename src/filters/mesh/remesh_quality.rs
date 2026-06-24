//! Remeshing with quality targets: target edge length, angle bounds.

use crate::data::{CellArray, Points, PolyData};

/// Remesh to achieve a target edge length by splitting long edges and
/// collapsing short edges.
pub fn remesh_to_target_length(mesh: &PolyData, target_length: f64, iterations: usize) -> PolyData {
    let mut pts: Vec<[f64; 3]> = (0..mesh.points.len()).map(|i| mesh.points.get(i)).collect();
    let mut tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter_map(|c| {
            if c.len() == 3 {
                Some([c[0] as usize, c[1] as usize, c[2] as usize])
            } else {
                None
            }
        })
        .collect();

    let hi = target_length * 1.33;
    let lo = target_length * 0.67;

    for _ in 0..iterations {
        // Split long edges
        let mut new_tris = Vec::new();
        let mut split_happened = false;
        for tri in &tris {
            let lens = [
                edge_len(&pts, tri[0], tri[1]),
                edge_len(&pts, tri[1], tri[2]),
                edge_len(&pts, tri[2], tri[0]),
            ];
            let longest = if lens[0] >= lens[1] && lens[0] >= lens[2] {
                0
            } else if lens[1] >= lens[2] {
                1
            } else {
                2
            };

            if lens[longest] > hi {
                let (a, b) = match longest {
                    0 => (tri[0], tri[1]),
                    1 => (tri[1], tri[2]),
                    _ => (tri[2], tri[0]),
                };
                let c = match longest {
                    0 => tri[2],
                    1 => tri[0],
                    _ => tri[1],
                };
                let mid_idx = pts.len();
                pts.push(midpoint(&pts[a], &pts[b]));
                new_tris.push([a, mid_idx, c]);
                new_tris.push([mid_idx, b, c]);
                split_happened = true;
            } else {
                new_tris.push(*tri);
            }
        }
        tris = new_tris;

        // Collapse short edges
        let mut collapse_map: Vec<usize> = (0..pts.len()).collect();
        for tri in &tris {
            for i in 0..3 {
                let a = tri[i];
                let b = tri[(i + 1) % 3];
                let ra = collapse_map[a];
                let rb = collapse_map[b];
                if ra != rb && edge_len(&pts, ra, rb) < lo {
                    let mid = midpoint(&pts[ra], &pts[rb]);
                    pts[ra] = mid;
                    collapse_map[rb] = ra;
                    // Propagate
                    for m in collapse_map.iter_mut() {
                        if *m == rb {
                            *m = ra;
                        }
                    }
                }
            }
        }
        // Remap and remove degenerate
        tris = tris
            .iter()
            .map(|t| [collapse_map[t[0]], collapse_map[t[1]], collapse_map[t[2]]])
            .filter(|t| t[0] != t[1] && t[1] != t[2] && t[0] != t[2])
            .collect();

        // Laplacian smoothing pass
        let adj = build_adj_from_tris(&tris, pts.len());
        let mut new_pts = pts.clone();
        for i in 0..pts.len() {
            if adj[i].is_empty() {
                continue;
            }
            let mut avg = [0.0; 3];
            for &j in &adj[i] {
                for c in 0..3 {
                    avg[c] += pts[j][c];
                }
            }
            let k = adj[i].len() as f64;
            for c in 0..3 {
                new_pts[i][c] = pts[i][c] * 0.5 + (avg[c] / k) * 0.5;
            }
        }
        pts = new_pts;

        if !split_happened {
            break;
        }
    }

    // Compact: remove unused points
    let mut used = vec![false; pts.len()];
    for t in &tris {
        used[t[0]] = true;
        used[t[1]] = true;
        used[t[2]] = true;
    }
    let mut new_idx = vec![0usize; pts.len()];
    let mut new_pts = Points::<f64>::new();
    for i in 0..pts.len() {
        if used[i] {
            new_idx[i] = new_pts.len();
            new_pts.push(pts[i]);
        }
    }
    let mut polys = CellArray::new();
    for t in &tris {
        polys.push_cell(&[
            new_idx[t[0]] as i64,
            new_idx[t[1]] as i64,
            new_idx[t[2]] as i64,
        ]);
    }

    let mut result = PolyData::new();
    result.points = new_pts;
    result.polys = polys;
    result
}

fn edge_len(pts: &[[f64; 3]], a: usize, b: usize) -> f64 {
    ((pts[a][0] - pts[b][0]).powi(2)
        + (pts[a][1] - pts[b][1]).powi(2)
        + (pts[a][2] - pts[b][2]).powi(2))
    .sqrt()
}
fn midpoint(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        (a[0] + b[0]) / 2.0,
        (a[1] + b[1]) / 2.0,
        (a[2] + b[2]) / 2.0,
    ]
}
fn build_adj_from_tris(tris: &[[usize; 3]], n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for t in tris {
        for i in 0..3 {
            let a = t[i];
            let b = t[(i + 1) % 3];
            if a < n && b < n {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remesh_grid() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64 * 2.0, y as f64 * 2.0, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = remesh_to_target_length(&mesh, 1.5, 3);
        assert!(result.points.len() >= 3);
        assert!(result.polys.num_cells() > 0);
    }

    #[test]
    fn small_mesh() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = remesh_to_target_length(&mesh, 3.0, 5);
        assert!(result.polys.num_cells() > 1); // should split the large triangle
    }
}
