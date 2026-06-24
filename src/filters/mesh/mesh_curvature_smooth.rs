//! Curvature-weighted Laplacian smoothing (smooths flat areas more than curved).
use crate::data::{CellArray, Points, PolyData};

pub fn curvature_smooth(mesh: &PolyData, iterations: usize, lambda: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !adj[a].contains(&b) {
                    adj[a].push(b);
                }
                if !adj[b].contains(&a) {
                    adj[b].push(a);
                }
            }
        }
    }
    let mut positions: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            [p[0], p[1], p[2]]
        })
        .collect();
    for _ in 0..iterations {
        let mut curvatures = vec![0.0f64; n];
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let k = adj[i].len() as f64;
            let mut lap = [0.0, 0.0, 0.0];
            for &j in &adj[i] {
                lap[0] += positions[j][0] - positions[i][0];
                lap[1] += positions[j][1] - positions[i][1];
                lap[2] += positions[j][2] - positions[i][2];
            }
            curvatures[i] = (lap[0] * lap[0] + lap[1] * lap[1] + lap[2] * lap[2]).sqrt() / k;
        }
        let cmax = curvatures.iter().cloned().fold(0.0f64, f64::max).max(1e-15);
        let mut new_pos = positions.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let weight = lambda * (1.0 - curvatures[i] / cmax); // less smoothing at high curvature
            let k = adj[i].len() as f64;
            for d in 0..3 {
                let avg: f64 = adj[i].iter().map(|&j| positions[j][d]).sum::<f64>() / k;
                new_pos[i][d] += weight * (avg - positions[i][d]);
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
    fn test_curv_smooth() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.3],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = curvature_smooth(&mesh, 5, 0.5);
        assert_eq!(r.points.len(), 4);
    }
}
