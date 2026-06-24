//! Compute and store Laplacian spectrum (eigenvalue sequence).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn laplacian_spectrum(mesh: &PolyData, num_eigenvalues: usize, iterations: usize) -> Vec<f64> {
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
            eigenvalue = lv.iter().zip(v.iter()).map(|(l, vi)| l * vi).sum::<f64>();
            let norm = lv.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-15);
            v = lv.iter().map(|x| x / norm).collect();
        }
        eigenvalues.push(eigenvalue);
        eigvecs.push(v);
    }
    eigenvalues
}
pub fn spectral_gap(mesh: &PolyData, iterations: usize) -> f64 {
    let spectrum = laplacian_spectrum(mesh, 2, iterations);
    if spectrum.len() >= 2 {
        (spectrum[1] - spectrum[0]).abs()
    } else {
        0.0
    }
}
pub fn attach_spectrum_as_field(
    mesh: &PolyData,
    num_eigenvalues: usize,
    iterations: usize,
) -> PolyData {
    let spectrum = laplacian_spectrum(mesh, num_eigenvalues, iterations);
    let n = mesh.points.len();
    // Store first eigenvalue as uniform scalar for visualization
    let val = spectrum.first().copied().unwrap_or(0.0);
    let data = vec![val; n];
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Lambda0", data, 1)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_spectrum() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let s = laplacian_spectrum(&m, 3, 30);
        assert_eq!(s.len(), 3);
    }
    #[test]
    fn test_gap() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let g = spectral_gap(&m, 30);
        assert!(g > 0.0);
    }
}
