//! Compute multiple Laplacian eigenvectors for spectral analysis.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn laplacian_basis(mesh: &PolyData, num_basis: usize, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }
    let nb = num_basis.min(n);
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !neighbors[a].contains(&b) {
                    neighbors[a].push(b);
                }
                if !neighbors[b].contains(&a) {
                    neighbors[b].push(a);
                }
            }
        }
    }
    let mut result = mesh.clone();
    let mut computed: Vec<Vec<f64>> = Vec::new();
    for bi in 0..nb {
        let mut v: Vec<f64> = (0..n)
            .map(|i| (i as f64 + bi as f64 * 0.37).sin())
            .collect();
        for _ in 0..iterations {
            // Apply Laplacian
            let mut lv = vec![0.0f64; n];
            for i in 0..n {
                if neighbors[i].is_empty() {
                    continue;
                }
                lv[i] = neighbors[i].len() as f64 * v[i];
                for &j in &neighbors[i] {
                    lv[i] -= v[j];
                }
            }
            // Remove constant component
            let mean = lv.iter().sum::<f64>() / n as f64;
            for x in &mut lv {
                *x -= mean;
            }
            // Orthogonalize against previous eigenvectors
            for prev in &computed {
                let dot: f64 = lv.iter().zip(prev).map(|(a, b)| a * b).sum();
                for j in 0..n {
                    lv[j] -= dot * prev[j];
                }
            }
            let norm = lv.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-15);
            v = lv.iter().map(|x| x / norm).collect();
        }
        let name = format!("Basis_{}", bi);
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(&name, v.clone(), 1)));
        computed.push(v);
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = laplacian_basis(&m, 3, 30);
        assert!(r.point_data().get_array("Basis_0").is_some());
        assert!(r.point_data().get_array("Basis_2").is_some());
    }
}
