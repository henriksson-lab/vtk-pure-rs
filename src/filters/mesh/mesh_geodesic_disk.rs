//! Extract geodesic disk (neighborhood within geodesic distance) from a vertex.
use crate::data::{CellArray, Points, PolyData};
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
        Some(self.cmp(other))
    }
}
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

pub fn geodesic_disk(mesh: &PolyData, seed: usize, max_distance: f64) -> PolyData {
    let n = mesh.points.len();
    if seed >= n || max_distance < 0.0 {
        return PolyData::new();
    }
    let mut nb: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            add_edge(mesh, cell[i], cell[(i + 1) % nc], &mut nb);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_edge(mesh, edge[0], edge[1], &mut nb);
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
        if cost > max_distance {
            break;
        }
        for &(next_node, weight) in &nb[node] {
            let next = cost + weight;
            if next < dist[next_node] {
                dist[next_node] = next;
                heap.push(State {
                    cost: next,
                    node: next_node,
                });
            }
        }
    }
    let reachable: std::collections::HashSet<usize> =
        (0..n).filter(|&i| dist[i] <= max_distance).collect();
    let mut used = vec![false; n];
    let mut kept = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.iter().all(|&v| reachable.contains(&(v as usize))) {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.to_vec());
        }
    }
    let mut pm = vec![0usize; n];
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        if used[i] {
            pm[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for c in &kept {
        polys.push_cell(&c.iter().map(|&v| pm[v as usize] as i64).collect::<Vec<_>>());
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
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
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 4, 3], [1, 3, 2]],
        );
        let r = geodesic_disk(&m, 0, 1.5);
        assert!(r.polys.num_cells() >= 1);
    }
}
