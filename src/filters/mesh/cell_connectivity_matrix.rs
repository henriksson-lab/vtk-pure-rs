use crate::data::PolyData;
use std::collections::HashMap;

/// Compute the cell adjacency matrix as a sparse list.
///
/// Two cells are adjacent if they share at least one edge.
/// Returns (cell_i, cell_j) pairs.
pub fn cell_adjacency_list(input: &PolyData) -> Vec<(usize, usize)> {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        if c.len() < 2 {
            continue;
        }
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut adj = Vec::new();
    for faces in edge_faces.values() {
        for i in 0..faces.len() {
            for j in i + 1..faces.len() {
                adj.push((faces[i].min(faces[j]), faces[i].max(faces[j])));
            }
        }
    }
    adj.sort();
    adj.dedup();
    adj
}

/// Compute average number of adjacent cells per cell (face adjacency).
pub fn average_cell_adjacency(input: &PolyData) -> f64 {
    let nc = input.polys.num_cells();
    if nc == 0 {
        return 0.0;
    }
    let adj = cell_adjacency_list(input);
    let mut degree = vec![0usize; nc];
    for &(a, b) in &adj {
        degree[a] += 1;
        degree[b] += 1;
    }
    degree.iter().sum::<usize>() as f64 / nc as f64
}

/// Compute the diameter of the face adjacency graph (longest shortest path).
pub fn face_graph_diameter(input: &PolyData) -> usize {
    let nc = input.polys.num_cells();
    if nc < 2 {
        return 0;
    }

    let adj = cell_adjacency_list(input);
    let mut graph: Vec<Vec<usize>> = vec![Vec::new(); nc];
    for &(a, b) in &adj {
        graph[a].push(b);
        graph[b].push(a);
    }

    let mut max_d = 0;
    for src in 0..nc {
        let mut dist = vec![usize::MAX; nc];
        dist[src] = 0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(src);
        while let Some(v) = queue.pop_front() {
            for &nb in &graph[v] {
                if dist[nb] == usize::MAX {
                    dist[nb] = dist[v] + 1;
                    queue.push_back(nb);
                }
            }
        }
        for &d in &dist {
            if d != usize::MAX && d > max_d {
                max_d = d;
            }
        }
    }
    max_d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacency_pair() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let adj = cell_adjacency_list(&pd);
        assert_eq!(adj.len(), 1); // one shared edge
    }

    #[test]
    fn average_adjacency() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let avg = average_cell_adjacency(&pd);
        assert_eq!(avg, 1.0); // each has 1 neighbor
    }

    #[test]
    fn diameter() {
        let mut pd = PolyData::new();
        for j in 0..3 {
            for i in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..2 {
            for i in 0..2 {
                let a = (j * 3 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 4]);
                pd.polys.push_cell(&[a, a + 4, a + 3]);
            }
        }

        let d = face_graph_diameter(&pd);
        assert!(d >= 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert!(cell_adjacency_list(&pd).is_empty());
        assert_eq!(average_cell_adjacency(&pd), 0.0);
    }
}
