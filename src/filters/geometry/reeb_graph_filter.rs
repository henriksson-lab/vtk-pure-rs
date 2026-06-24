//! Build a Reeb graph from a scalar field on a PolyData mesh.
//!
//! The filter sorts vertices by scalar value and sweeps through edges
//! to detect critical points (minima, maxima, saddles).

use std::collections::{HashMap, HashSet};

use crate::data::reeb_graph::{NodeType, ReebGraph};
use crate::data::PolyData;

/// Build a Reeb graph from a PolyData with a named scalar field.
///
/// The scalar field determines the height function for critical point detection.
/// Vertices are classified as:
/// - Minimum: all neighbors have higher scalar values
/// - Maximum: all neighbors have lower scalar values
/// - Saddle: mixed neighbor ordering (topology change)
///
/// Only vertices that are critical points become Reeb graph nodes.
/// Arcs connect adjacent critical points along monotone paths.
pub fn poly_data_to_reeb_graph(input: &PolyData, scalar_name: &str) -> ReebGraph {
    let n = input.points.len();
    if n == 0 {
        return ReebGraph::new();
    }

    // Get scalar values
    let scalars = match input.point_data().get_array(scalar_name) {
        Some(arr) => {
            let mut vals = vec![0.0f64; n];
            for i in 0..n {
                let mut buf = [0.0f64];
                arr.tuple_as_f64(i, &mut buf);
                vals[i] = buf[0];
            }
            vals
        }
        None => return ReebGraph::new(),
    };

    // Build adjacency from polygon cells
    let mut adj: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for cell in input.polys.iter() {
        let len = cell.len();
        for i in 0..len {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % len] as usize;
            adj[a].insert(b);
            adj[b].insert(a);
        }
    }
    // Also from line cells
    for cell in input.lines.iter() {
        for w in cell.windows(2) {
            let a = w[0] as usize;
            let b = w[1] as usize;
            adj[a].insert(b);
            adj[b].insert(a);
        }
    }

    // Classify each vertex
    let mut node_types = vec![NodeType::Regular; n];
    for v in 0..n {
        if adj[v].is_empty() {
            continue;
        }
        let sv = scalars[v];
        let mut lower = 0usize;
        let mut higher = 0usize;
        for &nb in &adj[v] {
            if scalars[nb] < sv {
                lower += 1;
            } else if scalars[nb] > sv {
                higher += 1;
            }
            // equal: treat as higher (tie-breaking by index)
        }
        // Handle ties by vertex index
        for &nb in &adj[v] {
            if (scalars[nb] - sv).abs() < f64::EPSILON {
                if nb > v {
                    higher += 1;
                } else {
                    lower += 1;
                }
            }
        }

        if lower == 0 {
            node_types[v] = NodeType::Minimum;
        } else if higher == 0 {
            node_types[v] = NodeType::Maximum;
        } else {
            // Check connected components of lower link
            let lower_neighbors: Vec<usize> = adj[v]
                .iter()
                .copied()
                .filter(|&nb| scalars[nb] < sv || (scalars[nb] == sv && nb < v))
                .collect();
            let upper_neighbors: Vec<usize> = adj[v]
                .iter()
                .copied()
                .filter(|&nb| scalars[nb] > sv || (scalars[nb] == sv && nb > v))
                .collect();

            // Count connected components of lower link
            let lower_components = count_link_components(&lower_neighbors, &adj);
            let upper_components = count_link_components(&upper_neighbors, &adj);

            if lower_components > 1 || upper_components > 1 {
                node_types[v] = NodeType::Saddle;
            }
        }
    }

    // Build Reeb graph: add critical points as nodes
    let mut rg = ReebGraph::new();
    let mut vertex_to_node: HashMap<usize, usize> = HashMap::new();

    // Sort vertices by scalar value
    let mut sorted: Vec<usize> = (0..n).collect();
    sorted.sort_by(|&a, &b| scalars[a].partial_cmp(&scalars[b]).unwrap().then(a.cmp(&b)));

    for &v in &sorted {
        if node_types[v] != NodeType::Regular {
            let nid = rg.add_node(v, scalars[v], node_types[v]);
            vertex_to_node.insert(v, nid);
        }
    }

    // Connect adjacent critical points with arcs
    // For each critical point, trace upward along edges to find next critical point
    let mut visited_arcs: HashSet<(usize, usize)> = HashSet::new();

    for &v in &sorted {
        if node_types[v] == NodeType::Regular {
            continue;
        }
        let src_node = vertex_to_node[&v];

        // Trace upward from v along each neighbor
        for &nb in &adj[v] {
            if scalars[nb] < scalars[v] || (scalars[nb] == scalars[v] && nb < v) {
                continue; // only go upward
            }

            let mut current = nb;
            let mut path = Vec::new();

            // Walk upward through regular vertices
            while node_types[current] == NodeType::Regular {
                path.push(current);
                // Find the upward neighbor (highest scalar among neighbors > current)
                let next = adj[current]
                    .iter()
                    .copied()
                    .filter(|&x| {
                        scalars[x] > scalars[current]
                            || (scalars[x] == scalars[current] && x > current)
                    })
                    .min_by(|&a, &b| scalars[a].partial_cmp(&scalars[b]).unwrap().then(a.cmp(&b)));

                match next {
                    Some(nx) => current = nx,
                    None => break,
                }
            }

            if node_types[current] != NodeType::Regular {
                let tgt_node = vertex_to_node[&current];
                let arc_key = (src_node.min(tgt_node), src_node.max(tgt_node));
                if visited_arcs.insert(arc_key) {
                    rg.add_arc(src_node, tgt_node, path);
                }
            }
        }
    }

    rg
}

/// Count connected components among a subset of vertices using their mutual adjacency.
fn count_link_components(vertices: &[usize], adj: &[HashSet<usize>]) -> usize {
    if vertices.is_empty() {
        return 0;
    }
    let vset: HashSet<usize> = vertices.iter().copied().collect();
    let mut visited = HashSet::new();
    let mut components = 0;

    for &start in vertices {
        if visited.contains(&start) {
            continue;
        }
        components += 1;
        let mut stack = vec![start];
        while let Some(v) = stack.pop() {
            if !visited.insert(v) {
                continue;
            }
            for &nb in &adj[v] {
                if vset.contains(&nb) && !visited.contains(&nb) {
                    stack.push(nb);
                }
            }
        }
    }
    components
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn reeb_graph_triangle() {
        // Single triangle with linear scalar: vertex 0=min, vertex 2=max
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let arr = DataArray::from_vec("height", vec![0.0, 1.0, 2.0], 1);
        pd.point_data_mut().add_array(AnyDataArray::F64(arr));

        let rg = poly_data_to_reeb_graph(&pd, "height");
        assert!(rg.num_nodes() >= 2);
        // Should have at least one minimum and one maximum
        assert!(!rg.nodes_of_type(NodeType::Minimum).is_empty());
        assert!(!rg.nodes_of_type(NodeType::Maximum).is_empty());
    }

    #[test]
    fn reeb_graph_two_triangles() {
        // Two triangles sharing an edge, with a clear min-to-max path
        //   3 (h=3)
        //  / \
        // 1---2  (h=1, h=2)
        //  \ /
        //   0 (h=0)
        let mut pd = PolyData::from_triangles(
            vec![
                [0.5, 0.0, 0.0], // 0
                [0.0, 1.0, 0.0], // 1
                [1.0, 1.0, 0.0], // 2
                [0.5, 2.0, 0.0], // 3
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let arr = DataArray::from_vec("height", vec![0.0, 1.0, 2.0, 3.0], 1);
        pd.point_data_mut().add_array(AnyDataArray::F64(arr));

        let rg = poly_data_to_reeb_graph(&pd, "height");
        assert!(rg.num_nodes() >= 2);
        assert!(rg.num_arcs() >= 1);
    }
}
