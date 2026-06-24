//! Geodesic convex hull on mesh surface.
use crate::data::{CellArray, Points, PolyData};
pub fn geodesic_convex_hull(mesh: &PolyData, seed_vertices: &[usize]) -> PolyData {
    let n = mesh.points.len();
    if seed_vertices.is_empty() || n == 0 {
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
    // Compute geodesic distance from all seeds (multi-source Dijkstra)
    let mut dist = vec![f64::INFINITY; n];
    let mut visited = vec![false; n];
    for &s in seed_vertices {
        if s < n {
            dist[s] = 0.0;
        }
    }
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
        if dist[u].is_infinite() {
            break;
        }
        visited[u] = true;
        for &(v, w) in &nb[u] {
            let alt = dist[u] + w;
            if alt < dist[v] {
                dist[v] = alt;
            }
        }
    }
    // Extract sub-mesh within geodesic hull (vertices reachable within max inter-seed distance)
    let max_inter: f64 = seed_vertices
        .iter()
        .flat_map(|&si| {
            seed_vertices.iter().map(move |&sj| {
                if si >= n || sj >= n || si == sj {
                    0.0
                } else {
                    let pa = mesh.points.get(si);
                    let pb = mesh.points.get(sj);
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt()
                }
            })
        })
        .fold(0.0f64, f64::max);
    let threshold = max_inter * 0.6;
    let mut used = vec![false; n];
    let mut kept = Vec::new();
    for cell in mesh.polys.iter() {
        let all_close = cell
            .iter()
            .all(|&v| (v as usize) < n && dist[v as usize] <= threshold);
        if all_close {
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
                [3.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2], [1, 4, 3]],
        );
        let r = geodesic_convex_hull(&m, &[0, 3]);
        assert!(r.polys.num_cells() >= 1);
    }
}
