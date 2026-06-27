use crate::data::{CellArray, Points, PolyData};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

/// Simplify a triangle mesh using quadric error metrics (Garland-Heckbert).
///
/// Iteratively collapses edges with the lowest quadric error until
/// `target_ratio` of the original face count remains (0.0-1.0).
/// More accurate than `quadric_clustering` for preserving shape.
pub fn edge_collapse_quadric(input: &PolyData, target_ratio: f64) -> PolyData {
    let target =
        ((input.polys.num_cells() as f64 * target_ratio.clamp(0.01, 1.0)).ceil() as usize).max(1);

    let n = input.points.len();
    let mut pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    let mut tris: Vec<[usize; 3]> = input
        .polys
        .iter()
        .filter(|c| c.len() >= 3 && c.iter().all(|&id| id >= 0 && (id as usize) < n))
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .filter(|tri| tri[0] != tri[1] && tri[1] != tri[2] && tri[0] != tri[2])
        .collect();

    let num_tris = tris.len();
    if num_tris == 0 || num_tris <= target {
        return input.clone();
    }

    // Compute per-vertex quadric matrices (symmetric 4x4 stored as 10 floats)
    let mut quadrics = vec![[0.0f64; 10]; n];
    for tri in &tris {
        let q = face_quadric(&pts[tri[0]], &pts[tri[1]], &pts[tri[2]]);
        for &v in tri {
            add_quadric(&mut quadrics[v], &q);
        }
    }

    // Build per-vertex adjacency: vertex -> set of triangle indices
    let mut adj: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for (ti, tri) in tris.iter().enumerate() {
        for &v in tri {
            adj[v].insert(ti);
        }
    }

    // Dead triangle bitmap
    let mut dead = vec![false; num_tris];

    // Version stamps per vertex (for lazy-deletion heap)
    let mut version: Vec<u64> = vec![0; n];

    // Build priority queue: (Reverse(cost), version_a, version_b, a, b)
    let mut heap: BinaryHeap<(Reverse<u64>, u64, u64, usize, usize)> = BinaryHeap::new();

    // Collect unique edges and seed the heap
    {
        let mut seen: HashSet<(usize, usize)> = HashSet::new();
        for tri in &tris {
            for k in 0..3 {
                let a = tri[k];
                let b = tri[(k + 1) % 3];
                let edge = if a < b { (a, b) } else { (b, a) };
                if seen.insert(edge) {
                    let cost = edge_cost(&quadrics[a], &quadrics[b], &pts[a], &pts[b]);
                    let cost_bits = cost_key(cost);
                    heap.push((
                        Reverse(cost_bits),
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

    while current_faces > target {
        let entry = match heap.pop() {
            Some(e) => e,
            None => break,
        };
        let (_, va, vb, a, b) = entry;

        // Stale entry check
        if version[a] != va || version[b] != vb {
            continue;
        }
        if a == b {
            continue;
        }

        // Collapse edge (a, b) -> a
        // Compute midpoint
        let mid = [
            (pts[a][0] + pts[b][0]) * 0.5,
            (pts[a][1] + pts[b][1]) * 0.5,
            (pts[a][2] + pts[b][2]) * 0.5,
        ];
        pts[a] = mid;

        // Merge quadrics
        let qb = quadrics[b];
        add_quadric(&mut quadrics[a], &qb);

        // Bump versions to invalidate stale heap entries
        version[a] += 1;
        version[b] += 1;

        // Find triangles shared by both a and b (these become degenerate)
        let shared: Vec<usize> = adj[a].intersection(&adj[b]).copied().collect();
        for &ti in &shared {
            if !dead[ti] {
                dead[ti] = true;
                current_faces -= 1;
                // Remove this triangle from adjacency of all its vertices
                for &v in &tris[ti] {
                    if v != a && v != b {
                        adj[v].remove(&ti);
                    }
                }
            }
        }

        // Update triangles that reference b to reference a instead
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
            // Check if triangle became degenerate after substitution
            if tri[0] == tri[1] || tri[1] == tri[2] || tri[0] == tri[2] {
                if !dead[ti] {
                    dead[ti] = true;
                    current_faces -= 1;
                    for &v in &[tri[0], tri[1], tri[2]] {
                        adj[v].remove(&ti);
                    }
                }
            } else {
                // Move triangle from b's adjacency to a's
                adj[a].insert(ti);
            }
        }
        adj[b].clear();

        // Collect neighbors of a and re-insert edges into the heap
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
            let cost_bits = cost_key(cost);
            heap.push((Reverse(cost_bits), version[a], version[nb], a, nb));
        }
    }

    // Remap and compact
    let mut pt_map = vec![usize::MAX; n];
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for (ti, tri) in tris.iter().enumerate() {
        if dead[ti] {
            continue;
        }
        let mapped: [i64; 3] = std::array::from_fn(|k| {
            let v = tri[k];
            if pt_map[v] == usize::MAX {
                let idx = out_points.len();
                pt_map[v] = idx;
                out_points.push(pts[v]);
            }
            pt_map[v] as i64
        });
        out_polys.push_cell(&mapped);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
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
    // Q = [a b c d]^T * [a b c d] stored as upper triangle
    let (a, b, c) = (n[0], n[1], n[2]);
    [
        a * a,
        a * b,
        a * c,
        a * d,
        b * b,
        b * c,
        b * d,
        c * c,
        c * d,
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
    fn reduces_face_count() {
        let mut pd = PolyData::new();
        for j in 0..5 {
            for i in 0..5 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..4 {
            for i in 0..4 {
                let a = (j * 5 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 6]);
                pd.polys.push_cell(&[a, a + 6, a + 5]);
            }
        }

        let result = edge_collapse_quadric(&pd, 0.5);
        assert!(result.polys.num_cells() < pd.polys.num_cells());
        assert!(result.polys.num_cells() > 0);
    }

    #[test]
    fn ratio_1_preserves() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = edge_collapse_quadric(&pd, 1.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = edge_collapse_quadric(&pd, 0.5);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn ignores_invalid_and_degenerate_triangles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 1]);

        let result = edge_collapse_quadric(&pd, 0.5);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn cost_key_orders_positive_costs() {
        let mut costs = vec![10.0, 0.5, 2.0, 1.0];
        costs.sort_by_key(|&cost| cost_key(cost));
        assert_eq!(costs, vec![0.5, 1.0, 2.0, 10.0]);
    }
}
