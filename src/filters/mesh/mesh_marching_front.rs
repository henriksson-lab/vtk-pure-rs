//! Marching front distance computation (fast marching-like on mesh graph).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn marching_front_distance(mesh: &PolyData, sources: &[usize]) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut nb: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    build_adjacency(mesh, &mut nb, n);
    let mut dist = vec![f64::INFINITY; n];
    let mut visited = vec![false; n];
    for &s in sources {
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
    let max_d = dist
        .iter()
        .filter(|&&d| d.is_finite())
        .cloned()
        .fold(0.0f64, f64::max);
    for d in &mut dist {
        if d.is_infinite() {
            *d = max_d;
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("FrontDist", dist, 1)));
    r.point_data_mut().set_active_scalars("FrontDist");
    r
}
pub fn marching_front_from_boundary(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    build_edge_counts(mesh, &mut ec, n);
    let sources: Vec<usize> = ec
        .iter()
        .filter(|(_, &c)| c == 1)
        .flat_map(|(&(a, b), _)| vec![a, b])
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    marching_front_distance(mesh, &sources)
}

fn build_adjacency(mesh: &PolyData, nb: &mut [Vec<(usize, f64)>], n: usize) {
    for cell in mesh.polys.iter() {
        for i in 0..cell.len() {
            add_weighted_edge(mesh, nb, n, cell[i], cell[(i + 1) % cell.len()]);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_weighted_edge(mesh, nb, n, edge[0], edge[1]);
        }
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            add_weighted_edge(mesh, nb, n, tri[0], tri[1]);
            add_weighted_edge(mesh, nb, n, tri[1], tri[2]);
            add_weighted_edge(mesh, nb, n, tri[2], tri[0]);
        }
    }
}

fn build_edge_counts(
    mesh: &PolyData,
    ec: &mut std::collections::HashMap<(usize, usize), usize>,
    n: usize,
) {
    for cell in mesh.polys.iter() {
        for i in 0..cell.len() {
            add_edge_count(ec, n, cell[i], cell[(i + 1) % cell.len()]);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_edge_count(ec, n, edge[0], edge[1]);
        }
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            add_edge_count(ec, n, tri[0], tri[1]);
            add_edge_count(ec, n, tri[1], tri[2]);
            add_edge_count(ec, n, tri[2], tri[0]);
        }
    }
}

fn add_weighted_edge(
    mesh: &PolyData,
    nb: &mut [Vec<(usize, f64)>],
    n: usize,
    a_id: i64,
    b_id: i64,
) {
    let Some(a) = valid_point_id(a_id, n) else {
        return;
    };
    let Some(b) = valid_point_id(b_id, n) else {
        return;
    };
    if a == b {
        return;
    }
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
    if !nb[a].iter().any(|&(x, _)| x == b) {
        nb[a].push((b, d));
        nb[b].push((a, d));
    }
}

fn add_edge_count(
    ec: &mut std::collections::HashMap<(usize, usize), usize>,
    n: usize,
    a_id: i64,
    b_id: i64,
) {
    let Some(a) = valid_point_id(a_id, n) else {
        return;
    };
    let Some(b) = valid_point_id(b_id, n) else {
        return;
    };
    if a == b {
        return;
    }
    *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
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
    fn test_sources() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = marching_front_distance(&m, &[0]);
        let arr = r.point_data().get_array("FrontDist").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 1e-10);
    }
    #[test]
    fn test_boundary() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = marching_front_from_boundary(&m);
        assert!(r.point_data().get_array("FrontDist").is_some());
    }
}
