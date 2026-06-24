//! Cotangent-weighted Laplacian smoothing.
use crate::data::PolyData;
pub fn smooth_cotangent(mesh: &PolyData, iterations: usize, lambda: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    for _ in 0..iterations {
        let mut lap = vec![[0.0f64; 3]; n];
        let mut wt = vec![0.0f64; n];
        for cell in &cells {
            if cell.len() != 3 {
                continue;
            }
            let ids = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
            let p = [pos[ids[0]], pos[ids[1]], pos[ids[2]]];
            for i in 0..3 {
                let j = (i + 1) % 3;
                let k = (i + 2) % 3;
                let eij = [p[j][0] - p[i][0], p[j][1] - p[i][1], p[j][2] - p[i][2]];
                let eik = [p[k][0] - p[i][0], p[k][1] - p[i][1], p[k][2] - p[i][2]];
                let dot = eij[0] * eik[0] + eij[1] * eik[1] + eij[2] * eik[2];
                let cross_l = ((eij[1] * eik[2] - eij[2] * eik[1]).powi(2)
                    + (eij[2] * eik[0] - eij[0] * eik[2]).powi(2)
                    + (eij[0] * eik[1] - eij[1] * eik[0]).powi(2))
                .sqrt();
                let cot = if cross_l > 1e-15 { dot / cross_l } else { 0.0 };
                let ejk = [p[k][0] - p[j][0], p[k][1] - p[j][1], p[k][2] - p[j][2]];
                lap[ids[j]][0] += cot * ejk[0];
                lap[ids[j]][1] += cot * ejk[1];
                lap[ids[j]][2] += cot * ejk[2];
                lap[ids[k]][0] -= cot * ejk[0];
                lap[ids[k]][1] -= cot * ejk[1];
                lap[ids[k]][2] -= cot * ejk[2];
                wt[ids[j]] += cot.abs();
                wt[ids[k]] += cot.abs();
            }
        }
        for i in 0..n {
            if wt[i] > 1e-15 {
                pos[i][0] += lambda * lap[i][0] / wt[i];
                pos[i][1] += lambda * lap[i][1] / wt[i];
                pos[i][2] += lambda * lap[i][2] / wt[i];
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
        let r = smooth_cotangent(&m, 3, 0.3);
        assert_eq!(r.points.len(), 4);
    }
}
