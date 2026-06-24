//! Laplacian smoothing that keeps boundary vertices fixed.
use crate::data::{Points, PolyData};

pub fn boundary_locked_smooth(mesh: &PolyData, iterations: usize, lambda: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut edge_count: std::collections::HashMap<(usize, usize), u32> =
        std::collections::HashMap::new();
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
                let e = if a < b { (a, b) } else { (b, a) };
                *edge_count.entry(e).or_insert(0) += 1;
            }
        }
    }
    let mut boundary = vec![false; n];
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            boundary[a] = true;
            boundary[b] = true;
        }
    }
    let mut positions: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            [p[0], p[1], p[2]]
        })
        .collect();
    for _ in 0..iterations {
        let mut new_pos = positions.clone();
        for i in 0..n {
            if boundary[i] || adj[i].is_empty() {
                continue;
            }
            let k = adj[i].len() as f64;
            for d in 0..3 {
                let avg: f64 = adj[i].iter().map(|&j| positions[j][d]).sum::<f64>() / k;
                new_pos[i][d] += lambda * (avg - positions[i][d]);
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
    fn test_locked_smooth() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.8, 0.1],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = boundary_locked_smooth(&mesh, 5, 0.5);
        // Boundary vertices should not move
        let p0 = r.points.get(0);
        assert_eq!(p0, [0.0, 0.0, 0.0]);
    }
}
