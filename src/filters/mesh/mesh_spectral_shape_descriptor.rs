//! Compute spectral shape descriptors (ShapeDNA / eigenvalue signature).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn shape_dna(mesh: &PolyData, num_eigenvalues: usize, iterations: usize) -> Vec<f64> {
    let n = mesh.points.len();
    if n < 3 {
        return vec![];
    }
    let ne = num_eigenvalues.min(n).max(1);
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
            }
        }
    }
    let mut eigenvalues = Vec::new();
    let mut eigvecs: Vec<Vec<f64>> = Vec::new();
    for ei in 0..ne {
        let mut v: Vec<f64> = (0..n)
            .map(|i| (i as f64 * 0.73 + ei as f64 * 1.37).sin())
            .collect();
        let mut eigenvalue = 0.0;
        for _ in 0..iterations {
            let mut lv = vec![0.0f64; n];
            for i in 0..n {
                if nb[i].is_empty() {
                    continue;
                }
                lv[i] = nb[i].len() as f64 * v[i];
                for &j in &nb[i] {
                    lv[i] -= v[j];
                }
            }
            let mean = lv.iter().sum::<f64>() / n as f64;
            for x in &mut lv {
                *x -= mean;
            }
            for prev in &eigvecs {
                let dot: f64 = lv.iter().zip(prev).map(|(a, b)| a * b).sum();
                for j in 0..n {
                    lv[j] -= dot * prev[j];
                }
            }
            let norm = lv.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-15);
            eigenvalue = lv.iter().zip(v.iter()).map(|(l, vi)| l * vi).sum::<f64>();
            v = lv.iter().map(|x| x / norm).collect();
        }
        eigenvalues.push(eigenvalue);
        eigvecs.push(v);
    }
    eigenvalues
}
pub fn attach_shape_dna(mesh: &PolyData, num_eigenvalues: usize, iterations: usize) -> PolyData {
    let eigenvalues = shape_dna(mesh, num_eigenvalues, iterations);
    let data: Vec<f64> = eigenvalues.iter().copied().collect();
    // Store as field data (one value per eigenvalue, not per vertex)
    // For simplicity, store first eigenvalue as uniform point scalar
    let n = mesh.points.len();
    let first = eigenvalues.first().copied().unwrap_or(0.0);
    let uniform: Vec<f64> = vec![first; n];
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ShapeDNA_0",
            uniform,
            1,
        )));
    r
}
pub fn shape_distance(dna_a: &[f64], dna_b: &[f64]) -> f64 {
    let n = dna_a.len().min(dna_b.len());
    if n == 0 {
        return f64::INFINITY;
    }
    (0..n)
        .map(|i| (dna_a[i] - dna_b[i]).powi(2))
        .sum::<f64>()
        .sqrt()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dna() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let dna = shape_dna(&m, 3, 30);
        assert_eq!(dna.len(), 3);
    }
    #[test]
    fn test_distance() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.1, 2.1, 3.1];
        let d = shape_distance(&a, &b);
        assert!(d < 0.3);
    }
    #[test]
    fn test_attach() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_shape_dna(&m, 2, 20);
        assert!(r.point_data().get_array("ShapeDNA_0").is_some());
    }
}
