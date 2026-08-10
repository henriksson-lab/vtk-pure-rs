//! Weighted Laplacian smoothing with per-vertex weight control.

use crate::data::{Points, PolyData};

/// Smooth with per-vertex weights from a scalar array.
///
/// Vertices with higher weight values are smoothed more: each vertex moves
/// toward the mean of its neighbours by `base_factor * weight[i]`, Jacobi
/// style (all vertices use the previous iteration's positions), as in
/// `vtkSmoothPolyDataFilter`'s relaxation step. Weights are clamped to
/// `[0, 1]`; vertices past the end of the weight array, or a missing/
/// multi-component array, get weight 1.0.
pub fn weighted_laplacian_smooth(
    mesh: &PolyData,
    weight_array: &str,
    base_factor: f64,
    iterations: usize,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let adj = build_adj(mesh, n);
    let weights = match mesh.point_data().get_array(weight_array) {
        Some(a) if a.num_components() == 1 => {
            let mut buf = [0.0f64];
            (0..n)
                .map(|i| {
                    if i < a.num_tuples() {
                        a.tuple_as_f64(i, &mut buf);
                        buf[0].clamp(0.0, 1.0)
                    } else {
                        1.0
                    }
                })
                .collect::<Vec<f64>>()
        }
        _ => vec![1.0; n],
    };

    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for _ in 0..iterations {
        let mut new_pos = pos.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let factor = base_factor * weights[i];
            let mut avg = [0.0; 3];
            for &j in &adj[i] {
                for c in 0..3 {
                    avg[c] += pos[j][c];
                }
            }
            let k = adj[i].len() as f64;
            for c in 0..3 {
                new_pos[i][c] = pos[i][c] * (1.0 - factor) + (avg[c] / k) * factor;
            }
        }
        pos = new_pos;
    }

    let mut result = mesh.clone();
    result.points = Points::from(pos);
    result
}

/// Smooth only vertices where a scalar exceeds a threshold.
pub fn conditional_smooth(
    mesh: &PolyData,
    condition_array: &str,
    threshold: f64,
    factor: f64,
    iterations: usize,
) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }
    let adj = build_adj(mesh, n);
    let should_smooth = match mesh.point_data().get_array(condition_array) {
        Some(a) if a.num_components() == 1 => {
            let mut buf = [0.0f64];
            (0..n)
                .map(|i| {
                    if i < a.num_tuples() {
                        a.tuple_as_f64(i, &mut buf);
                        buf[0] >= threshold
                    } else {
                        true
                    }
                })
                .collect::<Vec<bool>>()
        }
        _ => vec![true; n],
    };

    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for _ in 0..iterations {
        let mut new_pos = pos.clone();
        for i in 0..n {
            if !should_smooth[i] || adj[i].is_empty() {
                continue;
            }
            let mut avg = [0.0; 3];
            for &j in &adj[i] {
                for c in 0..3 {
                    avg[c] += pos[j][c];
                }
            }
            let k = adj[i].len() as f64;
            for c in 0..3 {
                new_pos[i][c] = pos[i][c] * (1.0 - factor) + (avg[c] / k) * factor;
            }
        }
        pos = new_pos;
    }

    let mut result = mesh.clone();
    result.points = Points::from(pos);
    result
}

fn build_adj(m: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut a: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for c in m.polys.iter() {
        let nc = c.len();
        for i in 0..nc {
            add_edge(&mut a, n, c[i], c[(i + 1) % nc]);
        }
    }
    for c in m.lines.iter() {
        for edge in c.windows(2) {
            add_edge(&mut a, n, edge[0], edge[1]);
        }
    }
    for c in m.strips.iter() {
        if c.len() < 3 {
            continue;
        }
        for i in 0..c.len() - 2 {
            let tri = if i % 2 == 0 {
                [c[i], c[i + 1], c[i + 2]]
            } else {
                [c[i + 1], c[i], c[i + 2]]
            };
            add_edge(&mut a, n, tri[0], tri[1]);
            add_edge(&mut a, n, tri[1], tri[2]);
            add_edge(&mut a, n, tri[2], tri[0]);
        }
    }
    a.into_iter().map(|s| s.into_iter().collect()).collect()
}

fn add_edge(adj: &mut [std::collections::HashSet<usize>], n: usize, a_id: i64, b_id: i64) {
    if a_id < 0 || b_id < 0 {
        return;
    }
    let a = a_id as usize;
    let b = b_id as usize;
    if a >= n || b >= n || a == b {
        return;
    }
    adj[a].insert(b);
    adj[b].insert(a);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn short_weight_array_uses_available_weights() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 1.0],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "w",
                vec![0.0, 0.0, 0.0],
                1,
            )));

        let r = weighted_laplacian_smooth(&m, "w", 0.5, 1);

        for i in 0..3 {
            assert_eq!(r.points.get(i), m.points.get(i));
        }
        assert_ne!(r.points.get(3), m.points.get(3));
    }

    #[test]
    fn weighted() {
        let mut pts = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, if x == 2 && y == 2 { 1.0 } else { 0.0 }]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mut mesh = PolyData::from_triangles(pts, tris);
        let w: Vec<f64> = (0..25).map(|i| if i == 12 { 1.0 } else { 0.0 }).collect();
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("w", w, 1)));
        let result = weighted_laplacian_smooth(&mesh, "w", 0.5, 5);
        let z = result.points.get(12)[2];
        assert!(z < 1.0); // bump vertex should be smoothed
    }
    #[test]
    fn conditional() {
        let mut pts = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, if x == 2 && y == 2 { 1.0 } else { 0.0 }]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mut mesh = PolyData::from_triangles(pts, tris);
        let cond: Vec<f64> = (0..25).map(|i| if i == 12 { 1.0 } else { 0.0 }).collect();
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("c", cond, 1)));
        let result = conditional_smooth(&mesh, "c", 0.5, 0.5, 5);
        assert_eq!(result.points.len(), 25);
    }
}
