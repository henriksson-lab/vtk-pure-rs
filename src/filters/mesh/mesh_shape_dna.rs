//! Shape DNA: Laplacian eigenvalue sequence as shape descriptor.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn shape_dna(mesh: &PolyData, n_eigenvalues: usize, power_iters: usize) -> Vec<f64> {
    let n = mesh.points.len();
    if n < 2 {
        return vec![];
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
    let neig = n_eigenvalues.min(n).max(1);
    let iters = power_iters.max(30);
    let mut eigenvalues = Vec::new();
    let mut deflation_vecs: Vec<Vec<f64>> = Vec::new();
    for _ in 0..neig {
        // Power iteration on Laplacian
        let mut v: Vec<f64> = (0..n).map(|i| (i as f64 * 0.1).sin() + 0.5).collect();
        for _ in 0..iters {
            // Apply Laplacian
            let mut lv = vec![0.0f64; n];
            for i in 0..n {
                lv[i] = adj[i].len() as f64 * v[i] - adj[i].iter().map(|&j| v[j]).sum::<f64>();
            }
            // Deflate against previous eigenvectors
            for prev in &deflation_vecs {
                let dot: f64 = lv.iter().zip(prev.iter()).map(|(a, b)| a * b).sum();
                for i in 0..n {
                    lv[i] -= dot * prev[i];
                }
            }
            // Normalize
            let norm = lv.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 1e-15 {
                for x in &mut lv {
                    *x /= norm;
                }
            }
            v = lv;
        }
        // Eigenvalue = Rayleigh quotient
        let mut lv = vec![0.0f64; n];
        for i in 0..n {
            lv[i] = adj[i].len() as f64 * v[i] - adj[i].iter().map(|&j| v[j]).sum::<f64>();
        }
        let lambda: f64 = v.iter().zip(lv.iter()).map(|(a, b)| a * b).sum();
        eigenvalues.push(lambda);
        deflation_vecs.push(v);
    }
    eigenvalues
}

pub fn shape_dna_as_data(mesh: &PolyData, n_eigenvalues: usize, power_iters: usize) -> PolyData {
    let eigs = shape_dna(mesh, n_eigenvalues, power_iters);
    let n = mesh.points.len();
    let mut result = mesh.clone();
    if !eigs.is_empty() {
        let data = vec![eigs[0]; n]; // store first eigenvalue as scalar
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "ShapeDNA_0",
                data,
                1,
            )));
        result.point_data_mut().set_active_scalars("ShapeDNA_0");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_shape_dna() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let eigs = shape_dna(&mesh, 3, 50);
        assert_eq!(eigs.len(), 3);
    }
}
