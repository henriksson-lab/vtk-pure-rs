use crate::data::PolyData;

/// Compute the first k eigenvalues of the graph Laplacian.
///
/// The eigenvalues describe the spectral shape of the mesh. Useful for shape
/// matching and retrieval.
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

    let mut lap = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        lap[i][i] = adj[i].len() as f64;
        for &j in &adj[i] {
            lap[i][j] -= 1.0;
        }
    }

    let mut eigenvalues = jacobi_eigenvalues(lap);
    eigenvalues.sort_by(|a, b| a.partial_cmp(b).unwrap());
    eigenvalues.truncate(k);
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

fn jacobi_eigenvalues(mut a: Vec<Vec<f64>>) -> Vec<f64> {
    let n = a.len();
    if n == 0 {
        return vec![];
    }
    let max_iter = 50 * n * n;
    for _ in 0..max_iter {
        let mut p = 0usize;
        let mut q = 1usize.min(n - 1);
        let mut max_off = 0.0f64;
        for i in 0..n {
            for j in i + 1..n {
                let v = a[i][j].abs();
                if v > max_off {
                    max_off = v;
                    p = i;
                    q = j;
                }
            }
        }
        if max_off < 1e-12 {
            break;
        }

        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];
        let tau = (aqq - app) / (2.0 * apq);
        let t = if tau >= 0.0 {
            1.0 / (tau + (1.0 + tau * tau).sqrt())
        } else {
            -1.0 / (-tau + (1.0 + tau * tau).sqrt())
        };
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;

        for r in 0..n {
            if r != p && r != q {
                let arp = a[r][p];
                let arq = a[r][q];
                let new_rp = c * arp - s * arq;
                let new_rq = s * arp + c * arq;
                a[r][p] = new_rp;
                a[p][r] = new_rp;
                a[r][q] = new_rq;
                a[q][r] = new_rq;
            }
        }

        a[p][p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        a[q][q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        a[p][q] = 0.0;
        a[q][p] = 0.0;
    }

    (0..n)
        .map(|i| if a[i][i].abs() < 1e-10 { 0.0 } else { a[i][i] })
        .collect()
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
        assert!(evals[0] < 1e-10); // connected graph has one zero eigenvalue
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
