//! Tutte embedding: maps a disk-topology mesh to a planar convex polygon.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn tutte_embedding(mesh: &PolyData, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
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
    // Find boundary vertices (edges shared by only one face)
    let mut edge_count = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let e = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(e).or_insert(0u32) += 1;
        }
    }
    let boundary_edges: Vec<(usize, usize)> = edge_count
        .iter()
        .filter(|(_, &c)| c == 1)
        .map(|(&e, _)| e)
        .collect();
    // Chain boundary vertices
    let mut boundary = Vec::new();
    if !boundary_edges.is_empty() {
        let mut bset: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for &(a, b) in &boundary_edges {
            bset.insert(a);
            bset.insert(b);
        }
        boundary = bset.into_iter().collect::<Vec<_>>();
        boundary.sort();
    }
    if boundary.is_empty() {
        boundary = (0..n.min(4)).collect();
    }
    // Map boundary to unit circle
    let nb = boundary.len();
    let mut u = vec![0.0f64; n];
    let mut v = vec![0.0f64; n];
    let is_boundary = {
        let mut b = vec![false; n];
        for (i, &vi) in boundary.iter().enumerate() {
            if vi < n {
                b[vi] = true;
                let angle = 2.0 * std::f64::consts::PI * i as f64 / nb as f64;
                u[vi] = angle.cos();
                v[vi] = angle.sin();
            }
        }
        b
    };
    // Iterative solve: interior vertices = average of neighbors
    for _ in 0..iterations.max(50) {
        for i in 0..n {
            if is_boundary[i] || adj[i].is_empty() {
                continue;
            }
            let k = adj[i].len() as f64;
            u[i] = adj[i].iter().map(|&j| u[j]).sum::<f64>() / k;
            v[i] = adj[i].iter().map(|&j| v[j]).sum::<f64>() / k;
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("U", u, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("V", v, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tutte() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.0],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = tutte_embedding(&mesh, 50);
        assert!(r.point_data().get_array("U").is_some());
        assert!(r.point_data().get_array("V").is_some());
    }
}
