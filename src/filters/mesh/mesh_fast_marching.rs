//! Fast marching method for geodesic distance on triangle meshes (simplified).
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn fast_marching(mesh: &PolyData, source: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || source >= n {
        return mesh.clone();
    }
    let mut adj: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
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
                if !adj[a].iter().any(|&(nb, _)| nb == b) {
                    adj[a].push((b, d));
                    adj[b].push((a, d));
                }
            }
        }
    }
    let mut dist = vec![f64::INFINITY; n];
    dist[source] = 0.0;
    let mut fixed = vec![false; n];
    // Dijkstra-like sweep (simplified fast marching)
    for _ in 0..n {
        // Find unfixed vertex with smallest distance
        let mut min_d = f64::INFINITY;
        let mut min_v = None;
        for i in 0..n {
            if !fixed[i] && dist[i] < min_d {
                min_d = dist[i];
                min_v = Some(i);
            }
        }
        let v = match min_v {
            Some(v) => v,
            None => break,
        };
        fixed[v] = true;
        for &(nb, w) in &adj[v] {
            let new_d = dist[v] + w;
            if new_d < dist[nb] {
                dist[nb] = new_d;
            }
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FMDistance",
            dist,
            1,
        )));
    result.point_data_mut().set_active_scalars("FMDistance");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fm() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = fast_marching(&mesh, 0);
        let arr = r.point_data().get_array("FMDistance").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 0.0);
        arr.tuple_as_f64(1, &mut b);
        assert!((b[0] - 1.0).abs() < 1e-9);
    }
}
