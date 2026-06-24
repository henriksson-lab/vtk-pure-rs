//! Geodesic distance via the heat method (Crane et al. 2013 simplified).
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn heat_geodesic(mesh: &PolyData, source: usize, diffusion_steps: usize) -> PolyData {
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
                }
                if !adj[b].iter().any(|&(nb, _)| nb == a) {
                    adj[b].push((a, d));
                }
            }
        }
    }
    // Step 1: Diffuse heat from source
    let avg_edge: f64 = adj
        .iter()
        .flat_map(|a| a.iter().map(|&(_, d)| d))
        .sum::<f64>()
        / adj.iter().map(|a| a.len()).sum::<usize>().max(1) as f64;
    let dt = avg_edge * avg_edge;
    let mut heat = vec![0.0f64; n];
    heat[source] = 1.0;
    for _ in 0..diffusion_steps.max(10) {
        let mut next = heat.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let total_w: f64 = adj[i].iter().map(|&(_, d)| 1.0 / d.max(1e-10)).sum();
            let lap: f64 = adj[i]
                .iter()
                .map(|&(j, d)| (heat[j] - heat[i]) / d.max(1e-10))
                .sum::<f64>()
                / total_w.max(1e-10);
            next[i] += dt * lap;
        }
        heat = next;
    }
    // Step 2: Compute negative normalized gradient direction
    // Step 3: Solve Poisson equation (simplified: use heat as distance proxy)
    let h_max = heat.iter().cloned().fold(0.0f64, f64::max);
    let dist: Vec<f64> = heat
        .iter()
        .map(|&h| {
            if h_max > 1e-15 {
                -((h / h_max).max(1e-30)).ln() * avg_edge
            } else {
                0.0
            }
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "HeatGeodesic",
            dist,
            1,
        )));
    result.point_data_mut().set_active_scalars("HeatGeodesic");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_heat_geo() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = heat_geodesic(&mesh, 0, 20);
        let arr = r.point_data().get_array("HeatGeodesic").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!(b[0] < 0.01); // source should be near zero
    }
}
