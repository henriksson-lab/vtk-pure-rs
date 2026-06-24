//! Geodesic distance on mesh using Dijkstra's algorithm.
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

pub fn geodesic_distance_dijkstra(mesh: &PolyData, seed: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || seed >= n {
        return mesh.clone();
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
    let mut dist = vec![f64::INFINITY; n];
    dist[seed] = 0.0;
    let mut heap = BinaryHeap::new();
    heap.push(State {
        cost: 0.0,
        node: seed,
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
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GeodesicDistance",
            dist,
            1,
        )));
    result
        .point_data_mut()
        .set_active_scalars("GeodesicDistance");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_geodesic() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = geodesic_distance_dijkstra(&mesh, 0);
        let arr = r.point_data().get_array("GeodesicDistance").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 0.0);
        arr.tuple_as_f64(1, &mut b);
        assert!((b[0] - 1.0).abs() < 1e-9);
    }
}
