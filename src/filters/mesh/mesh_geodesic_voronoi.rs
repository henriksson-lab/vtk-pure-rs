//! Geodesic Voronoi partition on mesh using multiple seed vertices.
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

/// Geodesic Voronoi partition: assign every vertex to the nearest seed along
/// mesh edges (Dijkstra over poly and line connectivity, edge weights are
/// Euclidean edge lengths).
///
/// Adds "VoronoiRegion" (seed index, -1 where unreachable, active scalars) and
/// "VoronoiDist" (geodesic distance to that seed, 0 where unreachable).
pub fn geodesic_voronoi(mesh: &PolyData, seeds: &[usize]) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || seeds.is_empty() {
        return mesh.clone();
    }
    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            add_edge(mesh, cell[i], cell[(i + 1) % nc], &mut adj);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_edge(mesh, edge[0], edge[1], &mut adj);
        }
    }
    let mut dist = vec![f64::INFINITY; n];
    let mut labels = vec![usize::MAX; n];
    let mut heap = BinaryHeap::new();
    for (si, &seed) in seeds.iter().enumerate() {
        if seed < n {
            dist[seed] = 0.0;
            labels[seed] = si;
            heap.push(State {
                cost: 0.0,
                node: seed,
            });
        }
    }
    while let Some(State { cost, node }) = heap.pop() {
        if cost > dist[node] {
            continue;
        }
        for &(nb, w) in &adj[node] {
            let next = cost + w;
            if next < dist[nb] {
                dist[nb] = next;
                labels[nb] = labels[node];
                heap.push(State {
                    cost: next,
                    node: nb,
                });
            }
        }
    }
    let label_data: Vec<f64> = labels
        .iter()
        .map(|&l| if l == usize::MAX { -1.0 } else { l as f64 })
        .collect();
    let dist_data: Vec<f64> = dist
        .iter()
        .map(|&d| if d.is_finite() { d } else { 0.0 })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VoronoiRegion",
            label_data,
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VoronoiDist",
            dist_data,
            1,
        )));
    result.point_data_mut().set_active_scalars("VoronoiRegion");
    result
}

fn add_edge(mesh: &PolyData, a: i64, b: i64, adj: &mut [Vec<(usize, f64)>]) {
    let n = adj.len();
    let Some(a) = valid_point_id(a, n) else {
        return;
    };
    let Some(b) = valid_point_id(b, n) else {
        return;
    };
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
    if !adj[a].iter().any(|&(v, _)| v == b) {
        adj[a].push((b, d));
    }
    if !adj[b].iter().any(|&(v, _)| v == a) {
        adj[b].push((a, d));
    }
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_voronoi() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [3.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = geodesic_voronoi(&mesh, &[0, 3]);
        let arr = r.point_data().get_array("VoronoiRegion").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 0.0);
        arr.tuple_as_f64(3, &mut b);
        assert_eq!(b[0], 1.0);
    }

    #[test]
    fn line_cell_edges_are_used() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        let result = geodesic_voronoi(&pd, &[0]);
        let arr = result.point_data().get_array("VoronoiRegion").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
        let dist = result.point_data().get_array("VoronoiDist").unwrap();
        dist.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 2.0);
    }

    #[test]
    fn unreachable_vertices_are_unlabelled() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([50.0, 50.0, 50.0]); // no cell references this one
        pd.polys.push_cell(&[0, 1, 2]);

        let result = geodesic_voronoi(&pd, &[0]);
        let arr = result.point_data().get_array("VoronoiRegion").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], -1.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = geodesic_voronoi(&pd, &[0]);
        assert_eq!(result.points.len(), 0);
    }
}
