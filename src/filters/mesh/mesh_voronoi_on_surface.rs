//! Voronoi partition on mesh surface from seed vertices.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn voronoi_on_mesh(mesh: &PolyData, seeds: &[usize]) -> PolyData {
    let n = mesh.points.len();
    if seeds.is_empty() || n == 0 {
        return mesh.clone();
    }
    // Dijkstra from all seeds simultaneously
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
    for strip in mesh.strips.iter() {
        for (i, tri) in strip.windows(3).enumerate() {
            if i % 2 == 0 {
                add_triangle_edges(mesh, tri[0], tri[1], tri[2], &mut nb);
            } else {
                add_triangle_edges(mesh, tri[1], tri[0], tri[2], &mut nb);
            }
        }
    }
    let mut dist = vec![f64::INFINITY; n];
    let mut label = vec![usize::MAX; n];
    let mut visited = vec![false; n];
    let mut have_seed = false;
    for (si, &seed) in seeds.iter().enumerate() {
        if seed < n {
            dist[seed] = 0.0;
            label[seed] = si;
            have_seed = true;
        }
    }
    if !have_seed {
        return mesh.clone();
    }
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
        if dist[u].is_infinite() {
            break;
        }
        visited[u] = true;
        for &(v, w) in &nb[u] {
            let alt = dist[u] + w;
            if alt < dist[v] {
                dist[v] = alt;
                label[v] = label[u];
            }
        }
    }
    let data: Vec<f64> = label
        .iter()
        .map(|&l| if l == usize::MAX { -1.0 } else { l as f64 })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VoronoiRegion",
            data,
            1,
        )));
    r.point_data_mut().set_active_scalars("VoronoiRegion");
    r
}

fn add_edge(mesh: &PolyData, a: i64, b: i64, nb: &mut [Vec<(usize, f64)>]) {
    let Some(a) = valid_point_id(a, nb.len()) else {
        return;
    };
    let Some(b) = valid_point_id(b, nb.len()) else {
        return;
    };
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
    if !nb[a].iter().any(|&(x, _)| x == b) {
        nb[a].push((b, d));
    }
    if !nb[b].iter().any(|&(x, _)| x == a) {
        nb[b].push((a, d));
    }
}

fn add_triangle_edges(mesh: &PolyData, a: i64, b: i64, c: i64, nb: &mut [Vec<(usize, f64)>]) {
    add_edge(mesh, a, b, nb);
    add_edge(mesh, b, c, nb);
    add_edge(mesh, c, a, nb);
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
                [4.0, 0.0, 0.0],
                [2.0, 4.0, 0.0],
                [4.0, 4.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = voronoi_on_mesh(&m, &[0, 3]);
        let arr = r.point_data().get_array("VoronoiRegion").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
