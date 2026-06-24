//! Cotangent-weighted Laplacian smoothing.
use crate::data::{Points, PolyData};

pub fn cotan_smooth(mesh: &PolyData, iterations: usize, lambda: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    let mut positions: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            [p[0], p[1], p[2]]
        })
        .collect();
    for _ in 0..iterations {
        let mut lap = vec![[0.0f64; 3]; n];
        let mut weight_sum = vec![0.0f64; n];
        for &[a, b, c] in &tris {
            if a >= n || b >= n || c >= n {
                continue;
            }
            let pa = positions[a];
            let pb = positions[b];
            let pc = positions[c];
            for &(i, j, k) in &[(a, b, c), (b, c, a), (c, a, b)] {
                let u = [
                    positions[j][0] - positions[k][0],
                    positions[j][1] - positions[k][1],
                    positions[j][2] - positions[k][2],
                ];
                let v = [
                    positions[i][0] - positions[k][0],
                    positions[i][1] - positions[k][1],
                    positions[i][2] - positions[k][2],
                ];
                let dot = u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
                let cross_len = ((u[1] * v[2] - u[2] * v[1]).powi(2)
                    + (u[2] * v[0] - u[0] * v[2]).powi(2)
                    + (u[0] * v[1] - u[1] * v[0]).powi(2))
                .sqrt();
                let w = if cross_len > 1e-15 {
                    (dot / cross_len).clamp(-100.0, 100.0)
                } else {
                    0.0
                };
                for d in 0..3 {
                    lap[i][d] += w * (positions[j][d] - positions[i][d]);
                }
                weight_sum[i] += w.abs();
            }
        }
        let mut new_pos = positions.clone();
        for i in 0..n {
            if weight_sum[i] > 1e-15 {
                for d in 0..3 {
                    new_pos[i][d] += lambda * lap[i][d] / weight_sum[i];
                }
            }
        }
        positions = new_pos;
    }
    let mut pts = Points::<f64>::new();
    for p in &positions {
        pts.push(*p);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = mesh.polys.clone();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cotan_smooth() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.3],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = cotan_smooth(&mesh, 3, 0.5);
        assert_eq!(r.points.len(), 4);
    }
}
