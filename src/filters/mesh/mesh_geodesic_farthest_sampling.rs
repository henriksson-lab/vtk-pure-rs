//! Farthest point sampling on mesh using geodesic distance.
use crate::data::{CellArray, Points, PolyData};
pub fn farthest_point_sample(mesh: &PolyData, num_samples: usize, seed: usize) -> Vec<usize> {
    let n = mesh.points.len();
    if n == 0 || num_samples == 0 {
        return vec![];
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
    let mut selected = vec![seed.min(n - 1)];
    let mut min_dist = vec![f64::INFINITY; n];
    for _ in 1..num_samples.min(n) {
        // Update distances from latest selected point
        let latest = *selected.last().unwrap();
        let mut dist = vec![f64::INFINITY; n];
        dist[latest] = 0.0;
        let mut visited = vec![false; n];
        for _ in 0..n {
            let u = (0..n).filter(|&i| !visited[i]).min_by(|&a, &b| {
                dist[a]
                    .partial_cmp(&dist[b])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let u = match u {
                Some(u) => u,
                None => break,
            };
            visited[u] = true;
            for &(v, w) in &nb[u] {
                let alt = dist[u] + w;
                if alt < dist[v] {
                    dist[v] = alt;
                }
            }
        }
        for i in 0..n {
            min_dist[i] = min_dist[i].min(dist[i]);
        }
        // Pick farthest
        let farthest = (0..n).filter(|i| !selected.contains(i)).max_by(|&a, &b| {
            min_dist[a]
                .partial_cmp(&min_dist[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        match farthest {
            Some(f) => selected.push(f),
            None => {
                break;
            }
        }
    }
    selected
}
pub fn farthest_point_sample_polydata(
    mesh: &PolyData,
    num_samples: usize,
    seed: usize,
) -> PolyData {
    let indices = farthest_point_sample(mesh, num_samples, seed);
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for &i in &indices {
        let idx = pts.len();
        pts.push(mesh.points.get(i));
        verts.push_cell(&[idx as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
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
                [3.0, 0.0, 0.0],
                [1.5, 3.0, 0.0],
                [3.0, 3.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let s = farthest_point_sample(&m, 3, 0);
        assert_eq!(s.len(), 3);
        assert_eq!(s[0], 0);
    }
    #[test]
    fn test_polydata() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = farthest_point_sample_polydata(&m, 2, 0);
        assert_eq!(r.points.len(), 2);
    }
}
