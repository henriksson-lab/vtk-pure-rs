use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::{HashMap, HashSet};

/// For each vertex, count the number of neighboring vertices (connected by an edge).
///
/// Adds a "VertexNeighborCount" point data array (1-component, i32) to the output.
pub fn compute_vertex_neighbor_counts(input: &PolyData) -> PolyData {
    let npts = input.points.len();
    let adj = build_adjacency(input);

    let counts: Vec<f64> = (0..npts)
        .map(|i| adj.get(&i).map_or(0, |s| s.len()) as f64)
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VertexNeighborCount",
            counts,
            1,
        )));
    pd
}

/// Extract the N-ring neighborhood of a given vertex as a set of vertex indices.
///
/// The 1-ring consists of all vertices directly connected to the given vertex by an edge.
/// The N-ring is obtained by expanding outward N times.
/// The center vertex itself is NOT included in the result.
pub fn n_ring_neighborhood(input: &PolyData, vertex: usize, n: usize) -> HashSet<usize> {
    let adj = build_adjacency(input);
    let mut current: HashSet<usize> = HashSet::new();
    current.insert(vertex);

    let mut all_visited: HashSet<usize> = HashSet::new();
    all_visited.insert(vertex);

    for _ring in 0..n {
        let mut next: HashSet<usize> = HashSet::new();
        for &v in &current {
            if let Some(neighbors) = adj.get(&v) {
                for &nb in neighbors {
                    if !all_visited.contains(&nb) {
                        next.insert(nb);
                        all_visited.insert(nb);
                    }
                }
            }
        }
        current = next;
    }

    all_visited.remove(&vertex);
    all_visited
}

/// Build an adjacency map: vertex index -> set of neighboring vertex indices.
fn build_adjacency(input: &PolyData) -> HashMap<usize, HashSet<usize>> {
    let mut adj: HashMap<usize, HashSet<usize>> = HashMap::new();

    for cell in input.polys.iter() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            adj.entry(a).or_default().insert(b);
            adj.entry(b).or_default().insert(a);
        }
    }
    adj
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_neighbor_counts() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_vertex_neighbor_counts(&pd);
        let arr = result
            .point_data()
            .get_array("VertexNeighborCount")
            .unwrap();
        assert_eq!(arr.num_tuples(), 3);
        let mut val = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut val);
            assert!(
                (val[0] - 2.0).abs() < 1e-10,
                "vertex {} should have 2 neighbors",
                i
            );
        }
    }

    #[test]
    fn fan_center_has_more_neighbors() {
        // Center vertex 0 connected to 4 outer vertices
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3], [0, 3, 4], [0, 4, 1]],
        );
        let result = compute_vertex_neighbor_counts(&pd);
        let arr = result
            .point_data()
            .get_array("VertexNeighborCount")
            .unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!(
            (val[0] - 4.0).abs() < 1e-10,
            "center vertex should have 4 neighbors"
        );
    }

    #[test]
    fn n_ring_neighborhood_two_rings() {
        // A strip of 4 triangles: 0-1-2-3-4 along x
        // triangles: (0,1,2), (1,3,2), (1,3,4) -- let's use a simple chain
        // vertices: 0,1,2,3,4,5
        // tri (0,1,2), tri (2,1,3), tri (2,3,4), tri (4,3,5)
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 1.0, 0.0],
                [4.0, 0.0, 0.0],
                [5.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [2, 1, 3], [2, 3, 4], [4, 3, 5]],
        );

        // 1-ring of vertex 0: {1, 2}
        let ring1 = n_ring_neighborhood(&pd, 0, 1);
        assert!(ring1.contains(&1));
        assert!(ring1.contains(&2));
        assert!(!ring1.contains(&0));
        assert_eq!(ring1.len(), 2);

        // 2-ring of vertex 0: {1, 2, 3, 4} (from 1 we reach 3; from 2 we reach 3,4)
        let ring2 = n_ring_neighborhood(&pd, 0, 2);
        assert!(ring2.contains(&3));
        assert!(ring2.contains(&4));
        assert_eq!(ring2.len(), 4);
    }
}
