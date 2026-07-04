//! Quadric error metric mesh simplification.

use crate::data::{CellArray, Points, PolyData};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

/// Simplify mesh using quadric error metrics to target face count.
pub fn simplify_quadric(mesh: &PolyData, target_faces: usize) -> PolyData {
    let n = mesh.points.len();
    let mut pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let mut tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3 && c.iter().all(|&id| id >= 0 && (id as usize) < n))
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .filter(|tri| tri[0] != tri[1] && tri[1] != tri[2] && tri[0] != tri[2])
        .collect();

    let num_tris = tris.len();
    if num_tris == 0 || num_tris <= target_faces {
        return mesh.clone();
    }

    // Compute per-vertex quadric matrices from incident face planes.
    let mut quadrics = vec![[0.0f64; 10]; n];
    for tri in &tris {
        let q = face_quadric(&pts[tri[0]], &pts[tri[1]], &pts[tri[2]]);
        for &v in tri {
            add_quadric(&mut quadrics[v], &q);
        }
    }

    let mut adj: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for (ti, tri) in tris.iter().enumerate() {
        for &v in tri {
            adj[v].insert(ti);
        }
    }

    let mut dead = vec![false; num_tris];
    let mut version = vec![0u64; n];
    let mut heap: BinaryHeap<(Reverse<u64>, u64, u64, usize, usize)> = BinaryHeap::new();

    {
        let mut seen: HashSet<(usize, usize)> = HashSet::new();
        for tri in &tris {
            for k in 0..3 {
                let a = tri[k];
                let b = tri[(k + 1) % 3];
                let edge = if a < b { (a, b) } else { (b, a) };
                if seen.insert(edge) {
                    let cost = edge_cost(&quadrics[a], &quadrics[b], &pts[a], &pts[b]);
                    heap.push((
                        Reverse(cost_key(cost)),
                        version[edge.0],
                        version[edge.1],
                        edge.0,
                        edge.1,
                    ));
                }
            }
        }
    }

    let mut current_faces = num_tris;
    while current_faces > target_faces {
        let Some((_, va, vb, a, b)) = heap.pop() else {
            break;
        };
        if version[a] != va || version[b] != vb || a == b {
            continue;
        }

        let mid = [
            (pts[a][0] + pts[b][0]) * 0.5,
            (pts[a][1] + pts[b][1]) * 0.5,
            (pts[a][2] + pts[b][2]) * 0.5,
        ];
        pts[a] = mid;

        let qb = quadrics[b];
        add_quadric(&mut quadrics[a], &qb);
        version[a] += 1;
        version[b] += 1;

        let shared: Vec<usize> = adj[a].intersection(&adj[b]).copied().collect();
        for &ti in &shared {
            if !dead[ti] {
                dead[ti] = true;
                current_faces -= 1;
                for &v in &tris[ti] {
                    if v != a && v != b {
                        adj[v].remove(&ti);
                    }
                }
            }
        }

        let b_tris: Vec<usize> = adj[b].iter().copied().collect();
        for ti in b_tris {
            if dead[ti] {
                continue;
            }
            let tri = &mut tris[ti];
            for v in tri.iter_mut() {
                if *v == b {
                    *v = a;
                }
            }
            if tri[0] == tri[1] || tri[1] == tri[2] || tri[0] == tri[2] {
                if !dead[ti] {
                    dead[ti] = true;
                    current_faces -= 1;
                    for &v in &[tri[0], tri[1], tri[2]] {
                        adj[v].remove(&ti);
                    }
                }
            } else {
                adj[a].insert(ti);
            }
        }
        adj[b].clear();

        let neighbors: Vec<usize> = {
            let mut nbrs = HashSet::new();
            for &ti in &adj[a] {
                if dead[ti] {
                    continue;
                }
                for &v in &tris[ti] {
                    if v != a {
                        nbrs.insert(v);
                    }
                }
            }
            nbrs.into_iter().collect()
        };

        for &nb in &neighbors {
            let cost = edge_cost(&quadrics[a], &quadrics[nb], &pts[a], &pts[nb]);
            heap.push((Reverse(cost_key(cost)), version[a], version[nb], a, nb));
        }
    }

    let mut pt_map = vec![usize::MAX; n];
    let mut new_pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for (ti, tri) in tris.iter().enumerate() {
        if dead[ti] {
            continue;
        }
        let mapped: [i64; 3] = std::array::from_fn(|k| {
            let v = tri[k];
            if pt_map[v] == usize::MAX {
                let id = new_pts.len();
                pt_map[v] = id;
                new_pts.push(pts[v]);
            }
            pt_map[v] as i64
        });
        polys.push_cell(&mapped);
    }

    let mut result = PolyData::new();
    result.points = new_pts;
    result.polys = polys;
    result
}

fn face_quadric(v0: &[f64; 3], v1: &[f64; 3], v2: &[f64; 3]) -> [f64; 10] {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let mut n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-15 {
        n[0] /= len;
        n[1] /= len;
        n[2] /= len;
    }
    let d = -(n[0] * v0[0] + n[1] * v0[1] + n[2] * v0[2]);
    [
        n[0] * n[0],
        n[0] * n[1],
        n[0] * n[2],
        n[0] * d,
        n[1] * n[1],
        n[1] * n[2],
        n[1] * d,
        n[2] * n[2],
        n[2] * d,
        d * d,
    ]
}

fn add_quadric(dest: &mut [f64; 10], src: &[f64; 10]) {
    for i in 0..10 {
        dest[i] += src[i];
    }
}

fn edge_cost(qa: &[f64; 10], qb: &[f64; 10], pa: &[f64; 3], pb: &[f64; 3]) -> f64 {
    let mid = [
        (pa[0] + pb[0]) * 0.5,
        (pa[1] + pb[1]) * 0.5,
        (pa[2] + pb[2]) * 0.5,
    ];
    let mut q = [0.0f64; 10];
    for i in 0..10 {
        q[i] = qa[i] + qb[i];
    }
    eval_quadric(&q, &mid)
}

fn eval_quadric(q: &[f64; 10], v: &[f64; 3]) -> f64 {
    let (x, y, z) = (v[0], v[1], v[2]);
    q[0] * x * x
        + 2.0 * q[1] * x * y
        + 2.0 * q[2] * x * z
        + 2.0 * q[3] * x
        + q[4] * y * y
        + 2.0 * q[5] * y * z
        + 2.0 * q[6] * y
        + q[7] * z * z
        + 2.0 * q[8] * z
        + q[9]
}

fn cost_key(cost: f64) -> u64 {
    let finite = if cost.is_finite() {
        cost.max(0.0)
    } else {
        f64::INFINITY
    };
    finite.to_bits()
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
