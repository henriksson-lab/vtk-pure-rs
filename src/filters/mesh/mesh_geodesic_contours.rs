//! Generate geodesic distance contours from a seed vertex.
use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
pub fn geodesic_contours(mesh: &PolyData, seed: usize, contour_spacing: f64) -> PolyData {
    let n = mesh.points.len();
    if seed >= n {
        return PolyData::new();
    }
    // Dijkstra
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
    dist[seed] = 0.0;
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
    // Attach distance as scalar
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GeodesicDist",
            dist.clone(),
            1,
        )));
    result.point_data_mut().set_active_scalars("GeodesicDist");
    // Extract contour lines
    let max_d = dist
        .iter()
        .filter(|&&d| d.is_finite())
        .cloned()
        .fold(0.0f64, f64::max);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut iso = contour_spacing;
    while iso < max_d {
        for cell in mesh.polys.iter() {
            if cell.len() < 3 {
                continue;
            }
            let nc = cell.len();
            let mut edge_pts: Vec<[f64; 3]> = Vec::new();
            for i in 0..nc {
                let a = cell[i] as usize;
                let b = cell[(i + 1) % nc] as usize;
                let da = dist[a];
                let db = dist[b];
                if da.is_finite() && db.is_finite() && (da - iso) * (db - iso) < 0.0 {
                    let t = (iso - da) / (db - da);
                    let pa = mesh.points.get(a);
                    let pb = mesh.points.get(b);
                    edge_pts.push([
                        pa[0] + t * (pb[0] - pa[0]),
                        pa[1] + t * (pb[1] - pa[1]),
                        pa[2] + t * (pb[2] - pa[2]),
                    ]);
                }
            }
            if edge_pts.len() == 2 {
                let i0 = pts.len();
                pts.push(edge_pts[0]);
                pts.push(edge_pts[1]);
                lines.push_cell(&[i0 as i64, (i0 + 1) as i64]);
            }
        }
        iso += contour_spacing;
    }
    let mut contour_mesh = PolyData::new();
    contour_mesh.points = pts;
    contour_mesh.lines = lines;
    contour_mesh
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
        let r = geodesic_contours(&m, 0, 1.0);
        assert!(r.lines.num_cells() >= 1);
    }
}
