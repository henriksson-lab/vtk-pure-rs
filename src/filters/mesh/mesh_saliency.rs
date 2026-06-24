//! Mesh saliency based on multi-scale mean curvature differences.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn mesh_saliency(mesh: &PolyData, scales: &[f64]) -> PolyData {
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
    // Compute mean curvature approximation (Laplacian magnitude)
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
    // Multi-scale Gaussian smoothing and difference
    let used_scales = if scales.is_empty() {
        vec![1.0, 2.0, 4.0]
    } else {
        scales.to_vec()
    };
    let mut saliency = vec![0.0f64; n];
    for &sigma in &used_scales {
        let iters = (sigma * 3.0).ceil() as usize;
        let mut smooth = curvature.clone();
        for _ in 0..iters {
            let mut next = smooth.clone();
            for i in 0..n {
                if adj[i].is_empty() {
                    continue;
                }
                let avg: f64 = adj[i].iter().map(|&j| smooth[j]).sum::<f64>() / adj[i].len() as f64;
                next[i] = 0.5 * smooth[i] + 0.5 * avg;
            }
            smooth = next;
        }
        for i in 0..n {
            saliency[i] += (curvature[i] - smooth[i]).abs();
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Saliency", saliency, 1,
        )));
    result.point_data_mut().set_active_scalars("Saliency");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_saliency() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = mesh_saliency(&mesh, &[1.0, 2.0]);
        assert!(r.point_data().get_array("Saliency").is_some());
    }
}
