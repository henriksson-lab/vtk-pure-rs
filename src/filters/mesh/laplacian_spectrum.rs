use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the first k approximate eigenvalues of the graph Laplacian.
///
/// Uses power iteration with deflation. The eigenvalues describe the
/// spectral shape of the mesh. Useful for shape matching and retrieval.
pub fn laplacian_eigenvalues(input: &PolyData, k: usize) -> Vec<f64> {
    let n = input.points.len();
    if n < 2 {
        return vec![];
    }
    let k = k.max(1).min(n);

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
            if !adj[b].contains(&a) {
                adj[b].push(a);
            }
        }
    }

    // Apply Laplacian: L*v = degree*v - A*v
    let apply_lap = |v: &[f64]| -> Vec<f64> {
        let mut lv = vec![0.0f64; n];
        for i in 0..n {
            lv[i] = adj[i].len() as f64 * v[i];
            for &j in &adj[i] {
                lv[i] -= v[j];
            }
        }
        lv
    };

    let mut eigenvalues = Vec::with_capacity(k);
    let mut eigenvectors: Vec<Vec<f64>> = Vec::new();

    for _ in 0..k {
        // Power iteration on L
        let s = 1.0 / (n as f64).sqrt();
        let mut v: Vec<f64> = (0..n)
            .map(|i| (i as f64 * 0.1 + 0.5).sin() * s + s)
            .collect();

        for _ in 0..100 {
            let mut lv = apply_lap(&v);
            // Orthogonalize against previous eigenvectors
            for ev in &eigenvectors {
                let dot: f64 = lv.iter().zip(ev).map(|(a, b)| a * b).sum();
                for i in 0..n {
                    lv[i] -= dot * ev[i];
                }
            }
            let norm: f64 = lv.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 1e-15 {
                for x in &mut lv {
                    *x /= norm;
                }
            }
            v = lv;
        }

        let lv = apply_lap(&v);
        let eigenvalue: f64 = v.iter().zip(lv.iter()).map(|(a, b)| a * b).sum();
        eigenvalues.push(eigenvalue);
        eigenvectors.push(v);
    }

    eigenvalues.sort_by(|a, b| a.partial_cmp(b).unwrap());
    eigenvalues
}

/// Compute the spectral gap (ratio of 2nd to 1st non-zero eigenvalue).
pub fn spectral_gap(input: &PolyData) -> f64 {
    let evals = laplacian_eigenvalues(input, 3);
    // Skip near-zero eigenvalues
    let nonzero: Vec<f64> = evals.into_iter().filter(|&e| e > 1e-6).collect();
    if nonzero.len() >= 2 {
        nonzero[1] / nonzero[0]
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eigenvalues_basic() {
        let mut pd = PolyData::new();
        for j in 0..3 {
            for i in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..2 {
            for i in 0..2 {
                let a = (j * 3 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 4]);
                pd.polys.push_cell(&[a, a + 4, a + 3]);
            }
        }

        let evals = laplacian_eigenvalues(&pd, 3);
        assert_eq!(evals.len(), 3);
        assert!(evals[0] >= 0.0); // Laplacian eigenvalues are non-negative
    }

    #[test]
    fn spectral_gap_exists() {
        let mut pd = PolyData::new();
        for j in 0..4 {
            for i in 0..4 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..3 {
            for i in 0..3 {
                let a = (j * 4 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 5]);
                pd.polys.push_cell(&[a, a + 5, a + 4]);
            }
        }

        let gap = spectral_gap(&pd);
        assert!(gap >= 0.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert!(laplacian_eigenvalues(&pd, 3).is_empty());
    }
}
