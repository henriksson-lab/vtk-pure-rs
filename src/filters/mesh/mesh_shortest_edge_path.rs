//! Find shortest edge path between two vertices.
use crate::data::{CellArray, Points, PolyData};
pub fn shortest_edge_path(mesh: &PolyData, start: usize, end: usize) -> PolyData {
    let n = mesh.points.len();
    if start >= n || end >= n {
        return PolyData::new();
    }
    let mut nb: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                let d =
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt();
                if !nb[a].iter().any(|&(x, _)| x == b) {
                    nb[a].push((b, d));
                    nb[b].push((a, d));
                }
            }
        }
    }
    let mut dist = vec![f64::INFINITY; n];
    let mut prev = vec![usize::MAX; n];
    let mut visited = vec![false; n];
    dist[start] = 0.0;
    for _ in 0..n {
        let u = (0..n).filter(|&i| !visited[i]).min_by(|&a, &b| {
            dist[a]
                .partial_cmp(&dist[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let u = match u {
            Some(u) => u,
            None => {
                break;
            }
        };
        if u == end {
            break;
        }
        visited[u] = true;
        for &(v, w) in &nb[u] {
            let alt = dist[u] + w;
            if alt < dist[v] {
                dist[v] = alt;
                prev[v] = u;
            }
        }
    }
    if dist[end].is_infinite() {
        return PolyData::new();
    }
    let mut path = vec![end];
    let mut cur = end;
    while cur != start {
        cur = prev[cur];
        if cur == usize::MAX {
            return PolyData::new();
        }
        path.push(cur);
    }
    path.reverse();
    let mut pts = Points::<f64>::new();
    let ids: Vec<i64> = path
        .iter()
        .map(|&v| {
            let i = pts.len();
            pts.push(mesh.points.get(v));
            i as i64
        })
        .collect();
    let mut lines = CellArray::new();
    lines.push_cell(&ids);
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
pub fn path_length(mesh: &PolyData, start: usize, end: usize) -> f64 {
    let path = shortest_edge_path(mesh, start, end);
    if path.points.len() < 2 {
        return f64::INFINITY;
    }
    let mut total = 0.0;
    for i in 0..path.points.len() - 1 {
        let a = path.points.get(i);
        let b = path.points.get(i + 1);
        total += ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
    }
    total
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_path() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let p = shortest_edge_path(&m, 0, 3);
        assert!(p.points.len() >= 2);
        assert_eq!(p.lines.num_cells(), 1);
    }
    #[test]
    fn test_length() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let d = path_length(&m, 0, 1);
        assert!((d - 1.0).abs() < 1e-10);
    }
}
