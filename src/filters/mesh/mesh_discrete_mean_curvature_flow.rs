//! Discrete mean curvature flow for mesh fairing.
use crate::data::PolyData;
pub fn mean_curvature_fairing(mesh: &PolyData, steps: usize, dt: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for _ in 0..steps {
        let mut lap = vec![[0.0f64; 3]; n];
        let mut area = vec![0.0f64; n];
        for c in &cells {
            if c.len() != 3 {
                continue;
            }
            let ids = [c[0] as usize, c[1] as usize, c[2] as usize];
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
                for d in 0..3 {
                    lap[ids[j]][d] += cot * ejk[d] * 0.5;
                    lap[ids[k]][d] -= cot * ejk[d] * 0.5;
                }
            }
            let ta = 0.5
                * ((p[1][0] - p[0][0]) * (p[2][1] - p[0][1])
                    - (p[1][1] - p[0][1]) * (p[2][0] - p[0][0]))
                    .abs();
            for &id in &ids {
                area[id] += ta / 3.0;
            }
        }
        for i in 0..n {
            if area[i] > 1e-15 {
                let a = area[i];
                pos[i][0] += dt * lap[i][0] / a;
                pos[i][1] += dt * lap[i][1] / a;
                pos[i][2] += dt * lap[i][2] / a;
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
        let r = mean_curvature_fairing(&m, 3, 0.01);
        assert_eq!(r.points.len(), 4);
    }
}
