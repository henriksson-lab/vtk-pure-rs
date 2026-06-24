use crate::data::PolyData;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Farthest point sampling on a mesh using geodesic distances.
///
/// Iteratively selects the vertex farthest from all previously selected
/// vertices, measured by edge-weighted Dijkstra distance. Returns indices.
pub fn geodesic_farthest_point_sampling(input: &PolyData, num_samples: usize) -> Vec<usize> {
    let n = input.points.len();
    if n == 0 {
        return vec![];
    }
    let k = num_samples.max(1).min(n);

    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            let pa = input.points.get(a);
            let pb = input.points.get(b);
            let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                .sqrt();
            if !adj[a].iter().any(|&(v, _)| v == b) {
                adj[a].push((b, d));
                adj[b].push((a, d));
            }
        }
    }

    let mut samples = Vec::with_capacity(k);
    let mut dist = vec![f64::MAX; n];

    // Start from vertex 0
    samples.push(0);
    update_distances(&adj, 0, &mut dist);

    for _ in 1..k {
        // Pick farthest vertex
        let farthest = (0..n)
            .max_by(|&a, &b| dist[a].partial_cmp(&dist[b]).unwrap_or(Ordering::Equal))
            .unwrap_or(0);
        samples.push(farthest);
        update_distances(&adj, farthest, &mut dist);
    }

    samples
}

fn update_distances(adj: &[Vec<(usize, f64)>], source: usize, dist: &mut [f64]) {
    #[derive(PartialEq)]
    struct S(f64, usize);
    impl Eq for S {}
    impl PartialOrd for S {
        fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for S {
        fn cmp(&self, o: &Self) -> Ordering {
            o.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
        }
    }

    let n = dist.len();
    let mut local_dist = vec![f64::MAX; n];
    local_dist[source] = 0.0;
    let mut heap = BinaryHeap::new();
    heap.push(S(0.0, source));

    while let Some(S(d, u)) = heap.pop() {
        if d > local_dist[u] {
            continue;
        }
        for &(v, w) in &adj[u] {
            let nd = d + w;
            if nd < local_dist[v] {
                local_dist[v] = nd;
                heap.push(S(nd, v));
            }
        }
    }

    // Update global distances (min with existing)
    for i in 0..n {
        dist[i] = dist[i].min(local_dist[i]);
    }
}

/// Compute the coverage radius: max distance from any point to nearest sample.
pub fn coverage_radius(input: &PolyData, samples: &[usize]) -> f64 {
    let n = input.points.len();
    if n == 0 || samples.is_empty() {
        return 0.0;
    }

    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            let pa = input.points.get(a);
            let pb = input.points.get(b);
            let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                .sqrt();
            if !adj[a].iter().any(|&(v, _)| v == b) {
                adj[a].push((b, d));
                adj[b].push((a, d));
            }
        }
    }

    let mut dist = vec![f64::MAX; n];
    for &s in samples {
        update_distances(&adj, s, &mut dist);
    }
    dist.iter().copied().fold(0.0f64, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fps_basic() {
        let mut pd = PolyData::new();
        for i in 0..10 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);
        pd.polys.push_cell(&[4, 5, 6]);
        pd.polys.push_cell(&[6, 7, 8]);
        pd.polys.push_cell(&[8, 9, 0]);

        let samples = geodesic_farthest_point_sampling(&pd, 3);
        assert_eq!(samples.len(), 3);
        // First sample is always 0
        assert_eq!(samples[0], 0);
    }

    #[test]
    fn coverage_decreases() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);

        let s2 = geodesic_farthest_point_sampling(&pd, 2);
        let s3 = geodesic_farthest_point_sampling(&pd, 3);
        let r2 = coverage_radius(&pd, &s2);
        let r3 = coverage_radius(&pd, &s3);
        assert!(r3 <= r2); // more samples = smaller coverage radius
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert!(geodesic_farthest_point_sampling(&pd, 5).is_empty());
    }
}
