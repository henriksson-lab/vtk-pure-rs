//! Geodesic Voronoi diagram on mesh surface.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn geodesic_voronoi(mesh: &PolyData, seeds: &[usize]) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || seeds.is_empty() {
        return mesh.clone();
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
    let mut label = vec![usize::MAX; n];
    let mut visited = vec![false; n];
    for (si, &seed) in seeds.iter().enumerate() {
        if seed < n {
            dist[seed] = 0.0;
            label[seed] = si;
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
                label[v] = label[u];
            }
        }
    }
    let data: Vec<f64> = label
        .iter()
        .map(|&l| if l == usize::MAX { -1.0 } else { l as f64 })
        .collect();
    let dist_data: Vec<f64> = dist
        .iter()
        .map(|&d| if d.is_finite() { d } else { 0.0 })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VoronoiRegion",
            data,
            1,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VoronoiDist",
            dist_data,
            1,
        )));
    r.point_data_mut().set_active_scalars("VoronoiRegion");
    r
}
pub fn geodesic_voronoi_boundaries(mesh: &PolyData, seeds: &[usize]) -> PolyData {
    if seeds.is_empty() {
        return PolyData::new();
    }
    let labeled = geodesic_voronoi(mesh, seeds);
    let Some(arr) = labeled.point_data().get_array("VoronoiRegion") else {
        return PolyData::new();
    };
    let mut buf = [0.0f64];
    let labels: Vec<usize> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] < 0.0 {
                usize::MAX
            } else {
                buf[0] as usize
            }
        })
        .collect();
    let mut pts = crate::data::Points::<f64>::new();
    let mut lines = crate::data::CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], labels.len()) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], labels.len()) else {
                continue;
            };
            if labels[a] != labels[b] {
                let ia = *pm.entry(a).or_insert_with(|| {
                    let i = pts.len();
                    pts.push(mesh.points.get(a));
                    i
                });
                let ib = *pm.entry(b).or_insert_with(|| {
                    let i = pts.len();
                    pts.push(mesh.points.get(b));
                    i
                });
                lines.push_cell(&[ia as i64, ib as i64]);
            }
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}

fn add_edge(mesh: &PolyData, a: i64, b: i64, nb: &mut [Vec<(usize, f64)>]) {
    let n = nb.len();
    let Some(a) = valid_point_id(a, n) else {
        return;
    };
    let Some(b) = valid_point_id(b, n) else {
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

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_voronoi() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [2.0, 4.0, 0.0],
                [4.0, 4.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = geodesic_voronoi(&m, &[0, 3]);
        let arr = r.point_data().get_array("VoronoiRegion").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
    #[test]
    fn test_boundaries() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [2.0, 4.0, 0.0],
                [4.0, 4.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = geodesic_voronoi_boundaries(&m, &[0, 3]);
        assert!(r.lines.num_cells() >= 1);
    }
}
