//! Spectral mesh processing: filter vertex positions in frequency domain.
use crate::data::PolyData;
pub fn spectral_low_pass(mesh: &PolyData, cutoff: usize, iterations: usize) -> PolyData {
    spectral_filter(mesh, cutoff, iterations, true)
}
pub fn spectral_high_pass(mesh: &PolyData, cutoff: usize, iterations: usize) -> PolyData {
    spectral_filter(mesh, cutoff, iterations, false)
}
fn spectral_filter(mesh: &PolyData, cutoff: usize, iterations: usize, low_pass: bool) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
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
    // Compute eigenvectors
    let ne = cutoff.min(n).max(1);
    let mut eigvecs: Vec<Vec<f64>> = Vec::new();
    for ei in 0..ne {
        let mut v: Vec<f64> = (0..n)
            .map(|i| (i as f64 * 0.73 + ei as f64 * 1.37).sin())
            .collect();
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
            v = lv.iter().map(|x| x / norm).collect();
        }
        eigvecs.push(v);
    }
    // Project coordinates onto eigenvectors and reconstruct
    let pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let mut new_pos = if low_pass {
        vec![[0.0f64; 3]; n]
    } else {
        pos.clone()
    };
    for ev in &eigvecs {
        let mut coeff = [0.0f64; 3];
        for i in 0..n {
            for d in 0..3 {
                coeff[d] += pos[i][d] * ev[i];
            }
        }
        if low_pass {
            for i in 0..n {
                for d in 0..3 {
                    new_pos[i][d] += coeff[d] * ev[i];
                }
            }
        } else {
            for i in 0..n {
                for d in 0..3 {
                    new_pos[i][d] -= coeff[d] * ev[i];
                }
            }
        }
    }
    let mut r = mesh.clone();
    for i in 0..n {
        r.points.set(i, new_pos[i]);
    }
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_low() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = spectral_low_pass(&m, 2, 30);
        assert_eq!(r.points.len(), 4);
    }
    #[test]
    fn test_high() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = spectral_high_pass(&m, 2, 30);
        assert_eq!(r.points.len(), 4);
    }
}
