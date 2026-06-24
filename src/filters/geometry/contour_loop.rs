//! ContourLoopExtraction — extract closed loops from line cells.

use crate::data::{CellArray, PolyData};
use std::collections::HashMap;

/// Extract closed loops from the line cells of a PolyData.
///
/// Builds an adjacency graph from line segments, then follows chains to find
/// closed loops. Each closed loop is output as a single polyline cell.
pub fn extract_contour_loops(input: &PolyData) -> PolyData {
    // Build adjacency: for each point, list of connected points
    let mut adj: HashMap<i64, Vec<i64>> = HashMap::new();

    for cell in input.lines.iter() {
        if cell.len() < 2 {
            continue;
        }
        // Each line cell is a segment (or polyline)
        for w in cell.windows(2) {
            let a = w[0];
            let b = w[1];
            adj.entry(a).or_default().push(b);
            adj.entry(b).or_default().push(a);
        }
    }

    let mut visited_edges: HashMap<(i64, i64), bool> = HashMap::new();
    let mut loops = Vec::new();

    // For each node with exactly 2 connections, try to trace a loop
    let nodes: Vec<i64> = adj.keys().copied().collect();
    for &start in &nodes {
        let neighbors = match adj.get(&start) {
            Some(n) => n,
            None => continue,
        };
        for &next in neighbors {
            let edge_key = if start < next {
                (start, next)
            } else {
                (next, start)
            };
            if visited_edges.contains_key(&edge_key) {
                continue;
            }

            // Try to trace a chain from start->next
            let mut chain = vec![start, next];
            visited_edges.insert(edge_key, true);
            let mut current = next;
            let mut prev = start;

            loop {
                let nbrs = match adj.get(&current) {
                    Some(n) => n,
                    None => break,
                };
                // Find next unvisited neighbor (not prev)
                let mut found = false;
                for &nb in nbrs {
                    if nb == prev {
                        continue;
                    }
                    let ek = if current < nb {
                        (current, nb)
                    } else {
                        (nb, current)
                    };
                    if visited_edges.contains_key(&ek) {
                        continue;
                    }
                    visited_edges.insert(ek, true);
                    if nb == start {
                        // Closed loop found!
                        chain.push(nb);
                        loops.push(chain.clone());
                        found = true;
                        break;
                    }
                    chain.push(nb);
                    prev = current;
                    current = nb;
                    found = true;
                    break;
                }
                if !found {
                    break;
                }
                // Check if we closed the loop
                if *chain.last().unwrap() == start {
                    break;
                }
            }
        }
    }

    let mut result = PolyData::new();
    result.points = input.points.clone();
    let mut out_lines = CellArray::new();
    for lp in &loops {
        out_lines.push_cell(lp);
    }
    result.lines = out_lines;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_single_loop() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);

        // Square loop as 4 line segments
        pd.lines.push_cell(&[0, 1]);
        pd.lines.push_cell(&[1, 2]);
        pd.lines.push_cell(&[2, 3]);
        pd.lines.push_cell(&[3, 0]);

        let result = extract_contour_loops(&pd);
        assert_eq!(result.lines.num_cells(), 1);
        let loop_cell = result.lines.cell(0);
        // Should be a closed loop: first == last
        assert_eq!(loop_cell.first(), loop_cell.last());
        assert_eq!(loop_cell.len(), 5); // 4 unique + closing
    }

    #[test]
    fn no_loops_from_open_chain() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);

        pd.lines.push_cell(&[0, 1]);
        pd.lines.push_cell(&[1, 2]);

        let result = extract_contour_loops(&pd);
        assert_eq!(result.lines.num_cells(), 0);
    }
}
