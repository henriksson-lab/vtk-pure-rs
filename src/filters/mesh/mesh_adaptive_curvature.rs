//! Adaptive mesh refinement based on local curvature.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn adaptive_curvature_metric(mesh: &PolyData, smoothing_iters: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !adj[a].contains(&b) {
                    adj[a].push(b);
                }
                if !adj[b].contains(&a) {
                    adj[b].push(a);
                }
            }
        }
    }
    // Discrete Laplacian magnitude as curvature proxy
    let mut curvature = vec![0.0f64; n];
    for i in 0..n {
        if adj[i].is_empty() {
            continue;
        }
        let p = mesh.points.get(i);
        let k = adj[i].len() as f64;
        let mut lap = [0.0, 0.0, 0.0];
        for &j in &adj[i] {
            let q = mesh.points.get(j);
            lap[0] += q[0] - p[0];
            lap[1] += q[1] - p[1];
            lap[2] += q[2] - p[2];
        }
        curvature[i] = (lap[0] * lap[0] + lap[1] * lap[1] + lap[2] * lap[2]).sqrt() / k;
    }
    // Smooth the curvature field
    for _ in 0..smoothing_iters {
        let mut next = curvature.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let avg: f64 = adj[i].iter().map(|&j| curvature[j]).sum::<f64>() / adj[i].len() as f64;
            next[i] = 0.5 * curvature[i] + 0.5 * avg;
        }
        curvature = next;
    }
    // Normalize to [0,1] range
    let cmax = curvature.iter().cloned().fold(0.0f64, f64::max);
    if cmax > 1e-15 {
        for c in &mut curvature {
            *c /= cmax;
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AdaptiveMetric",
            curvature,
            1,
        )));
    result.point_data_mut().set_active_scalars("AdaptiveMetric");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_adaptive() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = adaptive_curvature_metric(&mesh, 3);
        assert!(r.point_data().get_array("AdaptiveMetric").is_some());
    }
}
