use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute geodesic distance from a source vertex to all other vertices using
/// Dijkstra's algorithm on the mesh edge graph.
///
/// Edge weights are Euclidean distances between connected vertices. Adds a
/// "GeodesicDistance" scalar array to point data. Unreachable vertices get
/// `f64::INFINITY`.
pub fn geodesic_distance(input: &PolyData, source_vertex: usize) -> PolyData {
    let n: usize = input.points.len();

    // Build adjacency list with edge weights
    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        let len: usize = cell.len();
        for j in 0..len {
            let a: usize = cell[j] as usize;
            let b: usize = cell[(j + 1) % len] as usize;
            if a >= n || b >= n {
                continue;
            }
            let pa = input.points.get(a);
            let pb = input.points.get(b);
            let dx: f64 = pa[0] - pb[0];
            let dy: f64 = pa[1] - pb[1];
            let dz: f64 = pa[2] - pb[2];
            let dist: f64 = (dx * dx + dy * dy + dz * dz).sqrt();
            adj[a].push((b, dist));
            adj[b].push((a, dist));
        }
    }
    for cell in input.lines.iter() {
        for edge in cell.windows(2) {
            let a: usize = edge[0] as usize;
            let b: usize = edge[1] as usize;
            if a >= n || b >= n {
                continue;
            }
            let pa = input.points.get(a);
            let pb = input.points.get(b);
            let dx: f64 = pa[0] - pb[0];
            let dy: f64 = pa[1] - pb[1];
            let dz: f64 = pa[2] - pb[2];
            let dist: f64 = (dx * dx + dy * dy + dz * dz).sqrt();
            adj[a].push((b, dist));
            adj[b].push((a, dist));
        }
    }

    // Dijkstra
    let mut distances: Vec<f64> = vec![f64::INFINITY; n];
    if source_vertex < n {
        distances[source_vertex] = 0.0;
    }

    let mut heap: BinaryHeap<DijkstraNode> = BinaryHeap::new();
    if source_vertex < n {
        heap.push(DijkstraNode {
            cost: 0.0,
            vertex: source_vertex,
        });
    }

    while let Some(DijkstraNode { cost, vertex }) = heap.pop() {
        if cost > distances[vertex] {
            continue;
        }
        for &(neighbor, weight) in &adj[vertex] {
            let new_cost: f64 = cost + weight;
            if new_cost < distances[neighbor] {
                distances[neighbor] = new_cost;
                heap.push(DijkstraNode {
                    cost: new_cost,
                    vertex: neighbor,
                });
            }
        }
    }

    let mut output = input.clone();
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GeodesicDistance",
            distances,
            1,
        )));
    output
}

#[derive(Debug, Clone)]
struct DijkstraNode {
    cost: f64,
    vertex: usize,
}

impl PartialEq for DijkstraNode {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost && self.vertex == other.vertex
    }
}

impl Eq for DijkstraNode {}

impl PartialOrd for DijkstraNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DijkstraNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_line_mesh() -> PolyData {
        // Three points in a line: 0 -- 1 -- 2, each 1.0 apart
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]); // extra point to form triangles
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[1, 2, 3]);
        pd
    }

    fn make_square() -> PolyData {
        // Square: two triangles forming a quad
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd
    }

    #[test]
    fn source_vertex_has_zero_distance() {
        let mesh = make_line_mesh();
        let result = geodesic_distance(&mesh, 0);
        let arr = result.point_data().get_array("GeodesicDistance").unwrap();
        let mut buf: [f64; 1] = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-12);
    }

    #[test]
    fn distances_increase_along_path() {
        let mesh = make_line_mesh();
        let result = geodesic_distance(&mesh, 0);
        let arr = result.point_data().get_array("GeodesicDistance").unwrap();

        let mut d: Vec<f64> = Vec::new();
        let mut buf: [f64; 1] = [0.0];
        for i in 0..mesh.points.len() {
            arr.tuple_as_f64(i, &mut buf);
            d.push(buf[0]);
        }

        // Vertex 0 distance = 0
        assert!((d[0] - 0.0).abs() < 1e-12);
        // Vertex 1 should be 1.0 from vertex 0
        assert!((d[1] - 1.0).abs() < 1e-10);
        // Vertex 2 should be reachable via vertex 1 and vertex 3
        assert!(d[2] < f64::INFINITY);
        assert!(d[2] > 0.0);
    }

    #[test]
    fn square_diagonal_vs_edge() {
        let mesh = make_square();
        let result = geodesic_distance(&mesh, 0);
        let arr = result.point_data().get_array("GeodesicDistance").unwrap();

        let mut buf: [f64; 1] = [0.0];
        // Distance to vertex 1 (adjacent, edge length 1.0)
        arr.tuple_as_f64(1, &mut buf);
        let d1: f64 = buf[0];
        assert!((d1 - 1.0).abs() < 1e-10);

        // Distance to vertex 2 (diagonal, connected via edge 0-2)
        arr.tuple_as_f64(2, &mut buf);
        let d2: f64 = buf[0];
        let expected_diag: f64 = (2.0f64).sqrt();
        assert!((d2 - expected_diag).abs() < 1e-10);
    }
}
