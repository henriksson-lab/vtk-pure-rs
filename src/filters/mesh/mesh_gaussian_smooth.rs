//! Gaussian-weighted mesh smoothing.
use crate::data::PolyData;
pub fn gaussian_smooth(mesh: &PolyData, iterations: usize, sigma: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
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
    let s2 = 2.0 * sigma * sigma;
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for _ in 0..iterations {
        let prev = pos.clone();
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let pi = prev[i];
            let mut sum = [0.0, 0.0, 0.0];
            let mut wsum = 0.0;
            for &j in &nb[i] {
                let pj = prev[j];
                let d2 =
                    (pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2);
                let w = (-d2 / s2).exp();
                sum[0] += w * pj[0];
                sum[1] += w * pj[1];
                sum[2] += w * pj[2];
                wsum += w;
            }
            if wsum > 1e-15 {
                pos[i] = [sum[0] / wsum, sum[1] / wsum, sum[2] / wsum];
            }
        }
    }
    let mut r = mesh.clone();
    for i in 0..n {
        r.points.set(i, pos[i]);
    }
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = gaussian_smooth(&m, 3, 1.0);
        assert_eq!(r.points.len(), 4);
    }
}
