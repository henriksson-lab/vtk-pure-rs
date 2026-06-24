//! Find the geodesic centroid (vertex minimizing max geodesic distance).
use crate::data::{AnyDataArray, DataArray, PolyData};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(PartialEq)]
struct State {
    cost: f64,
    node: usize,
}
impl Eq for State {}
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.cost.partial_cmp(&self.cost)
    }
}
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

pub fn geodesic_centroid(mesh: &PolyData) -> (usize, PolyData) {
    let n = mesh.points.len();
    if n == 0 {
        return (0, mesh.clone());
    }
    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                let d =
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt();
                adj[a].push((b, d));
                adj[b].push((a, d));
            }
        }
    }
    let mut best_vertex = 0;
    let mut best_max_dist = f64::INFINITY;
    let mut eccentricity = vec![0.0f64; n];
    for src in 0..n {
        let mut dist = vec![f64::INFINITY; n];
        dist[src] = 0.0;
        let mut heap = BinaryHeap::new();
        heap.push(State {
            cost: 0.0,
            node: src,
        });
        while let Some(State { cost, node }) = heap.pop() {
            if cost > dist[node] {
                continue;
            }
            for &(nb, w) in &adj[node] {
                let next = cost + w;
                if next < dist[nb] {
                    dist[nb] = next;
                    heap.push(State {
                        cost: next,
                        node: nb,
                    });
                }
            }
        }
        let max_d = dist
            .iter()
            .filter(|&&d| d.is_finite())
            .cloned()
            .fold(0.0f64, f64::max);
        eccentricity[src] = max_d;
        if max_d < best_max_dist {
            best_max_dist = max_d;
            best_vertex = src;
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Eccentricity",
            eccentricity,
            1,
        )));
    result.point_data_mut().set_active_scalars("Eccentricity");
    (best_vertex, result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_centroid() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let (v, r) = geodesic_centroid(&mesh);
        assert!(v < 3);
        assert!(r.point_data().get_array("Eccentricity").is_some());
    }
}
