//! Decompose a ReebGraph into join tree and split tree.
//!
//! The join tree captures how components merge from minima upward,
//! and the split tree captures how components merge from maxima downward.

use crate::data::reeb_graph::ReebGraph;

/// Decompose a ReebGraph into a join tree and a split tree.
///
/// **Join tree**: For each node sorted by scalar ascending, connect to the
/// lowest adjacent node with a lower scalar value.
///
/// **Split tree**: For each node sorted by scalar descending, connect to the
/// highest adjacent node with a higher scalar value.
///
/// Returns `(join_tree, split_tree)`.
pub fn join_split_trees(graph: &ReebGraph) -> (ReebGraph, ReebGraph) {
    let n = graph.num_nodes();
    if n == 0 {
        return (ReebGraph::new(), ReebGraph::new());
    }

    // Build adjacency: for each node, find all neighbor node indices via arcs
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for arc in &graph.arcs {
        adj[arc.source].push(arc.target);
        adj[arc.target].push(arc.source);
    }

    // Sort node indices by scalar ascending for join tree
    let mut sorted_asc: Vec<usize> = (0..n).collect();
    sorted_asc.sort_by(|&a, &b| {
        graph
            .node(a)
            .scalar_value
            .partial_cmp(&graph.node(b).scalar_value)
            .unwrap()
    });

    // Build join tree
    let mut join_tree = ReebGraph::new();
    // Add all nodes (preserving indices)
    for i in 0..n {
        let node = graph.node(i);
        join_tree.add_node(node.vertex_id, node.scalar_value, node.node_type);
    }
    // For each node in ascending order, connect to lowest neighbor with lower scalar
    for &i in &sorted_asc {
        let sv = graph.node(i).scalar_value;
        let mut best: Option<usize> = None;
        let mut best_scalar = f64::MAX;
        for &nbr in &adj[i] {
            let ns = graph.node(nbr).scalar_value;
            if ns < sv && ns < best_scalar {
                best_scalar = ns;
                best = Some(nbr);
            }
        }
        if let Some(parent) = best {
            join_tree.add_arc(parent, i, vec![]);
        }
    }

    // Build split tree
    let mut split_tree = ReebGraph::new();
    for i in 0..n {
        let node = graph.node(i);
        split_tree.add_node(node.vertex_id, node.scalar_value, node.node_type);
    }
    // For each node in descending order, connect to highest neighbor with higher scalar
    for &i in sorted_asc.iter().rev() {
        let sv = graph.node(i).scalar_value;
        let mut best: Option<usize> = None;
        let mut best_scalar = f64::MIN;
        for &nbr in &adj[i] {
            let ns = graph.node(nbr).scalar_value;
            if ns > sv && ns > best_scalar {
                best_scalar = ns;
                best = Some(nbr);
            }
        }
        if let Some(parent) = best {
            split_tree.add_arc(i, parent, vec![]);
        }
    }

    (join_tree, split_tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::reeb_graph::NodeType;

    #[test]
    fn test_join_split_trees() {
        // Build a simple Y-shaped Reeb graph:
        // Two minima (0.0, 0.1) merge at a saddle (0.5) -> maximum (1.0)
        let mut rg = ReebGraph::new();
        let n0 = rg.add_node(0, 0.0, NodeType::Minimum); // 0
        let n1 = rg.add_node(1, 0.1, NodeType::Minimum); // 1
        let n2 = rg.add_node(2, 0.5, NodeType::Saddle); // 2
        let n3 = rg.add_node(3, 1.0, NodeType::Maximum); // 3
        rg.add_arc(n0, n2, vec![]);
        rg.add_arc(n1, n2, vec![]);
        rg.add_arc(n2, n3, vec![]);

        let (join, split) = join_split_trees(&rg);

        // Join tree: all 4 nodes
        assert_eq!(join.num_nodes(), 4);
        // Join tree connects each non-minimum node to its lowest lower neighbor:
        // node 2 (0.5) -> lowest lower neighbor is 0 (0.0) => arc 0->2
        // node 3 (1.0) -> lowest lower neighbor is 2 (0.5) => arc 2->3
        // node 0 and node 1 are minima, no lower neighbors
        assert_eq!(join.num_arcs(), 2);

        // Split tree: all 4 nodes
        assert_eq!(split.num_nodes(), 4);
        // Split tree connects each non-maximum node to its highest upper neighbor:
        // node 0 (0.0) -> highest upper neighbor is 2 (0.5) => arc 0->2
        // node 1 (0.1) -> highest upper neighbor is 2 (0.5) => arc 1->2
        // node 2 (0.5) -> highest upper neighbor is 3 (1.0) => arc 2->3
        assert_eq!(split.num_arcs(), 3);

        // Verify join tree: node 0 should have an arc to node 2
        let has_arc_0_2 = join.arcs.iter().any(|a| a.source == n0 && a.target == n2);
        assert!(has_arc_0_2, "Join tree should have arc 0->2");

        // Verify split tree: node 2 should have an arc to node 3
        let has_arc_2_3 = split.arcs.iter().any(|a| a.source == n2 && a.target == n3);
        assert!(has_arc_2_3, "Split tree should have arc 2->3");
    }
}
