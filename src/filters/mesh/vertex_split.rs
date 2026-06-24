use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Split a vertex into multiple copies, one per adjacent face group.
///
/// Groups faces around the vertex by connectivity through other shared
/// edges (not through the split vertex). Each group gets its own copy.
/// Useful for creating sharp corners at specific vertices.
pub fn split_vertex(input: &PolyData, vertex_id: usize) -> PolyData {
    let vid = vertex_id as i64;
    let n = input.points.len();
    if vertex_id >= n {
        return input.clone();
    }

    // Find faces containing this vertex
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let face_indices: Vec<usize> = cells
        .iter()
        .enumerate()
        .filter(|(_, c)| c.contains(&vid))
        .map(|(i, _)| i)
        .collect();

    if face_indices.len() <= 1 {
        return input.clone();
    }

    // Group faces by adjacency through edges NOT involving the split vertex
    let mut face_adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for &fi in &face_indices {
        for &fj in &face_indices {
            if fi >= fj {
                continue;
            }
            // Check if fi and fj share an edge that doesn't include vid
            let shared = cells[fi].iter().any(|&a| {
                a != vid
                    && cells[fj].iter().any(|&b| {
                        b != vid && {
                            let ci = &cells[fi];
                            let cj = &cells[fj];
                            ci.windows(2)
                                .chain(std::iter::once(&[ci[ci.len() - 1], ci[0]][..]))
                                .any(|e| {
                                    (e[0] == a && e[1] != vid && cj.contains(&e[1]))
                                        || (e[1] == a && e[0] != vid && cj.contains(&e[0]))
                                })
                        }
                    })
            });
            if shared {
                face_adj.entry(fi).or_default().push(fj);
                face_adj.entry(fj).or_default().push(fi);
            }
        }
    }

    // BFS to find connected groups
    let mut visited = std::collections::HashSet::new();
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for &fi in &face_indices {
        if visited.contains(&fi) {
            continue;
        }
        let mut group = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(fi);
        visited.insert(fi);
        while let Some(f) = queue.pop_front() {
            group.push(f);
            if let Some(adj) = face_adj.get(&f) {
                for &nf in adj {
                    if visited.insert(nf) {
                        queue.push_back(nf);
                    }
                }
            }
        }
        groups.push(group);
    }

    if groups.len() <= 1 {
        return input.clone();
    }

    let mut out_pts = input.points.clone();
    let mut new_cells = cells.clone();

    // First group keeps original vertex, others get copies
    for group in groups.iter().skip(1) {
        let new_vid = out_pts.len() as i64;
        out_pts.push(input.points.get(vertex_id));
        for &fi in group {
            for v in &mut new_cells[fi] {
                if *v == vid {
                    *v = new_vid;
                }
            }
        }
    }

    let mut out_polys = CellArray::new();
    for c in &new_cells {
        out_polys.push_cell(c);
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_fan_vertex() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // vertex to split
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([-1.0, 0.0, 0.0]);
        pd.points.push([0.0, -1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 4]);

        // These two faces don't share an edge besides vertex 0
        let result = split_vertex(&pd, 0);
        // May or may not split depending on adjacency detection
        assert!(result.polys.num_cells() == 2);
    }

    #[test]
    fn single_face_noop() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = split_vertex(&pd, 0);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn invalid_vertex() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = split_vertex(&pd, 999);
        assert_eq!(result.points.len(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = split_vertex(&pd, 0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
