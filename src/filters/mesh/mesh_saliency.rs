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
            let Ok(a) = usize::try_from(cell[i]) else {
                continue;
            };
            let Ok(b) = usize::try_from(cell[(i + 1) % nc]) else {
                continue;
            };
            if a >= n || b >= n {
                continue;
            }
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
            if !adj[b].contains(&a) {
                adj[b].push(a);
            }
        }
    }
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    // Compute mean curvature approximation (Laplacian magnitude)
    let mut curvature = vec![0.0f64; n];
    for i in 0..n {
        if adj[i].is_empty() {
            continue;
        }
        let p = pts[i];
        let k = adj[i].len() as f64;
        let mut lap = [0.0, 0.0, 0.0];
        for &j in &adj[i] {
            let q = pts[j];
            lap[0] += q[0] - p[0];
            lap[1] += q[1] - p[1];
            lap[2] += q[2] - p[2];
        }
        curvature[i] = (lap[0] * lap[0] + lap[1] * lap[1] + lap[2] * lap[2]).sqrt() / k;
    }

    let smooth = |sigma: f64| -> Vec<f64> {
        if sigma <= 0.0 {
            return curvature.clone();
        }
        let inv_2s2 = 1.0 / (2.0 * sigma * sigma);
        let mut result = curvature.clone();
        for _ in 0..3 {
            let mut next = result.clone();
            for i in 0..n {
                if adj[i].is_empty() {
                    continue;
                }
                let mut sum = result[i];
                let mut weight_sum = 1.0;
                for &j in &adj[i] {
                    let d2 = (pts[i][0] - pts[j][0]).powi(2)
                        + (pts[i][1] - pts[j][1]).powi(2)
                        + (pts[i][2] - pts[j][2]).powi(2);
                    let weight = (-d2 * inv_2s2).exp();
                    sum += weight * result[j];
                    weight_sum += weight;
                }
                next[i] = sum / weight_sum;
            }
            result = next;
        }
        result
    };

    // Multi-scale Gaussian smoothing and difference
    let used_scales = if scales.is_empty() {
        vec![1.0, 2.0, 4.0]
    } else {
        scales.to_vec()
    };
    let mut saliency = vec![0.0f64; n];
    for &sigma in &used_scales {
        let fine = smooth(sigma);
        let coarse = smooth(2.0 * sigma);
        for i in 0..n {
            saliency[i] += (fine[i] - coarse[i]).abs();
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
