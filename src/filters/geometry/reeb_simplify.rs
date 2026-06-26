//! Simplify a Reeb graph by removing low-persistence arcs.
//!
//! Persistence is the absolute difference in scalar values between arc endpoints.
//! Arcs with persistence below the threshold are removed and their endpoints merged.

use crate::data::reeb_graph::ReebGraph;

/// Simplify a Reeb graph by removing arcs whose persistence (|scalar_max - scalar_min|)
/// is below the given threshold.
///
/// When an arc is removed, its endpoints are merged into a single node (the one
/// with the lower scalar value is kept). Arcs connected to the removed node are
/// redirected to the surviving node.
pub fn simplify_reeb_graph(input: &ReebGraph, persistence_threshold: f64) -> ReebGraph {
    let n = input.num_nodes();
    if n == 0 {
        return ReebGraph::new();
    }

    // Union-find for merging nodes
    let mut parent: Vec<usize> = (0..n).collect();

    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }

    fn union(parent: &mut [usize], input: &ReebGraph, a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            // Keep the endpoint with the lower scalar value as representative.
            let sa = input.nodes[ra].scalar_value;
            let sb = input.nodes[rb].scalar_value;
            if sa < sb || (sa == sb && ra < rb) {
                parent[rb] = ra;
            } else {
                parent[ra] = rb;
            }
        }
    }

    // Identify arcs to remove (below persistence threshold)
    for arc in &input.arcs {
        let s0 = input.nodes[arc.source].scalar_value;
        let s1 = input.nodes[arc.target].scalar_value;
        let persistence = (s1 - s0).abs();
        if persistence < persistence_threshold {
            union(&mut parent, input, arc.source, arc.target);
        }
    }

    // Build mapping from old node index to new node index
    let mut rep_to_new: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut result = ReebGraph::new();

    for i in 0..n {
        let rep = find(&mut parent, i);
        if !rep_to_new.contains_key(&rep) {
            let node = &input.nodes[rep];
            let new_idx = result.add_node(node.vertex_id, node.scalar_value, node.node_type);
            rep_to_new.insert(rep, new_idx);
        }
    }

    // Re-add surviving arcs (those above threshold, with distinct endpoints after merging)
    let mut added_arcs: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();

    for arc in &input.arcs {
        let src_rep = find(&mut parent, arc.source);
        let tgt_rep = find(&mut parent, arc.target);
        if src_rep == tgt_rep {
            continue; // arc was collapsed
        }
        let new_src = rep_to_new[&src_rep];
        let new_tgt = rep_to_new[&tgt_rep];
        let key = (new_src.min(new_tgt), new_src.max(new_tgt));
        if added_arcs.insert(key) {
            result.add_arc(new_src, new_tgt, arc.vertex_ids.clone());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::reeb_graph::NodeType;

    #[test]
    fn simplify_removes_short_arcs() {
        let mut rg = ReebGraph::new();
        let n0 = rg.add_node(0, 0.0, NodeType::Minimum);
        let n1 = rg.add_node(1, 0.1, NodeType::Saddle); // small persistence from n0
        let n2 = rg.add_node(2, 5.0, NodeType::Maximum);

        rg.add_arc(n0, n1, vec![]);
        rg.add_arc(n1, n2, vec![]);

        // With threshold 0.5, the arc n0-n1 (persistence 0.1) should be removed
        let simplified = simplify_reeb_graph(&rg, 0.5);

        // n0 and n1 should be merged, leaving 2 nodes and 1 arc
        assert_eq!(simplified.num_nodes(), 2);
        assert_eq!(simplified.num_arcs(), 1);
    }
}
