//! Wave Kernel Signature (WKS) for shape description.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn wave_kernel_signature(
    mesh: &PolyData,
    energies: &[f64],
    sigma: f64,
    num_eigenvectors: usize,
    iterations: usize,
) -> PolyData {
    let n = mesh.points.len();
    if n < 3 || energies.is_empty() {
        return mesh.clone();
    }
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
    let ne = num_eigenvectors.min(n).max(1);
    let s2 = 2.0 * sigma * sigma;
    // Compute eigenvectors and approximate eigenvalues
    let mut eigvecs: Vec<Vec<f64>> = Vec::new();
    let mut eigvals = Vec::new();
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
        eigvals.push(eigenvalue);
        eigvecs.push(v);
    }
    // Compute WKS at each energy
    let mut result = mesh.clone();
    for (ei_idx, &energy) in energies.iter().enumerate() {
        let wks: Vec<f64> = (0..n)
            .map(|i| {
                let mut sum = 0.0;
                for k in 0..ne {
                    let phi_sq = eigvecs[k][i] * eigvecs[k][i];
                    let weight =
                        (-((energy.ln() - eigvals[k].abs().max(1e-30).ln()).powi(2)) / s2).exp();
                    sum += phi_sq * weight;
                }
                sum
            })
            .collect();
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                &format!("WKS_{}", ei_idx),
                wks,
                1,
            )));
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
        let r = wave_kernel_signature(&m, &[0.1, 1.0, 10.0], 1.0, 3, 30);
        assert!(r.point_data().get_array("WKS_0").is_some());
        assert!(r.point_data().get_array("WKS_2").is_some());
    }
}
