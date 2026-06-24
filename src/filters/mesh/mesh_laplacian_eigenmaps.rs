//! Laplacian eigenmaps for mesh embedding (dimensionality reduction).
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn laplacian_eigenmaps(mesh: &PolyData, n_dims: usize, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
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
    let dims = n_dims.min(3).max(1);
    let iters = iterations.max(50);
    let mut eigenvectors: Vec<Vec<f64>> = Vec::new();
    for d in 0..dims {
        let mut v: Vec<f64> = (0..n)
            .map(|i| ((i * (d + 1)) as f64 * 0.1).sin() + 0.3)
            .collect();
        for _ in 0..iters {
            let mut lv = vec![0.0f64; n];
            for i in 0..n {
                lv[i] = adj[i].len() as f64 * v[i] - adj[i].iter().map(|&j| v[j]).sum::<f64>();
            }
            // Deflate
            for prev in &eigenvectors {
                let dot: f64 = lv.iter().zip(prev.iter()).map(|(a, b)| a * b).sum();
                for i in 0..n {
                    lv[i] -= dot * prev[i];
                }
            }
            // Remove constant component
            let mean = lv.iter().sum::<f64>() / n as f64;
            for x in &mut lv {
                *x -= mean;
            }
            let norm = lv.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 1e-15 {
                for x in &mut lv {
                    *x /= norm;
                }
            }
            // Inverse iteration step
            let dot: f64 = v.iter().zip(lv.iter()).map(|(a, b)| a * b).sum();
            for i in 0..n {
                v[i] = v[i] - 0.1 * (lv[i] - dot * v[i]);
            }
            let mean = v.iter().sum::<f64>() / n as f64;
            for x in &mut v {
                *x -= mean;
            }
            let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 1e-15 {
                for x in &mut v {
                    *x /= norm;
                }
            }
        }
        eigenvectors.push(v);
    }
    let mut result = mesh.clone();
    for (d, ev) in eigenvectors.iter().enumerate() {
        let name = format!("Embed_{}", d);
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(&name, ev.clone(), 1)));
    }
    if !eigenvectors.is_empty() {
        result.point_data_mut().set_active_scalars("Embed_0");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_eigenmaps() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = laplacian_eigenmaps(&mesh, 2, 50);
        assert!(r.point_data().get_array("Embed_0").is_some());
        assert!(r.point_data().get_array("Embed_1").is_some());
    }
}
