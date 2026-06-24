//! Spectral mesh partitioning using the Fiedler vector (2nd smallest eigenvector of Laplacian).
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn spectral_partition(mesh: &PolyData, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 2 {
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
    // Power iteration for Fiedler vector: find eigenvector of L orthogonal to constant vector
    let iters = iterations.max(50);
    let mut v: Vec<f64> = (0..n).map(|i| (i as f64) / n as f64 - 0.5).collect();
    for _ in 0..iters {
        // Apply Laplacian: Lv[i] = deg(i)*v[i] - sum(v[j] for j in adj[i])
        let mut lv: Vec<f64> = vec![0.0; n];
        for i in 0..n {
            lv[i] = adj[i].len() as f64 * v[i] - adj[i].iter().map(|&j| v[j]).sum::<f64>();
        }
        // Project out constant vector
        let mean = lv.iter().sum::<f64>() / n as f64;
        for x in &mut lv {
            *x -= mean;
        }
        // Normalize
        let norm = lv.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 1e-15 {
            for x in &mut lv {
                *x /= norm;
            }
        }
        // For Fiedler vector we want smallest non-trivial eigenvector
        // Use inverse iteration approximation: v = v - (Lv projected)
        // Actually just use the Lv direction and subtract projection
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
    // Partition by sign of Fiedler vector
    let labels: Vec<f64> = v
        .iter()
        .map(|&x| if x >= 0.0 { 1.0 } else { 0.0 })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Partition",
            labels,
            1,
        )));
    result.point_data_mut().set_active_scalars("Partition");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_partition() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
                [3.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4], [3, 5, 4]],
        );
        let r = spectral_partition(&mesh, 100);
        assert!(r.point_data().get_array("Partition").is_some());
    }
}
