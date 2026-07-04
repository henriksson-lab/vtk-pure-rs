//! Tutte embedding: maps a disk-topology mesh to a planar convex polygon.
use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::{HashMap, HashSet};

pub fn tutte_embedding(mesh: &PolyData, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            if let Some((a, b)) = valid_edge(cell[i], cell[(i + 1) % nc], n) {
                let e = if a < b { (a, b) } else { (b, a) };
                *edge_count.entry(e).or_insert(0) += 1;
                if !adj[a].contains(&b) {
                    adj[a].push(b);
                }
                if !adj[b].contains(&a) {
                    adj[b].push(a);
                }
            }
        }
    }
    // Find and chain boundary vertices (edges shared by only one face).
    let mut boundary_adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            boundary_adj.entry(a).or_default().push(b);
            boundary_adj.entry(b).or_default().push(a);
        }
    }
    for neighbors in boundary_adj.values_mut() {
        neighbors.sort_unstable();
    }
    let mut boundary = Vec::new();
    if let Some(&start) = boundary_adj.keys().min() {
        let mut visited: HashSet<usize> = HashSet::new();
        let mut prev = None;
        let mut cur = start;
        loop {
            if !visited.insert(cur) {
                break;
            }
            boundary.push(cur);
            let next = boundary_adj.get(&cur).and_then(|neighbors| {
                neighbors
                    .iter()
                    .copied()
                    .find(|&v| Some(v) != prev && !visited.contains(&v))
                    .or_else(|| {
                        neighbors
                            .iter()
                            .copied()
                            .find(|&v| Some(v) != prev && v == start)
                    })
            });
            match next {
                Some(next) if next != start => {
                    prev = Some(cur);
                    cur = next;
                }
                _ => break,
            }
        }
    }
    if boundary.len() < 3 {
        return mesh.clone();
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
    // Iterative solve: interior vertices = average of previous neighbor positions.
    for _ in 0..iterations {
        let mut next_u = u.clone();
        let mut next_v = v.clone();
        for i in 0..n {
            if is_boundary[i] || adj[i].is_empty() {
                continue;
            }
            let k = adj[i].len() as f64;
            next_u[i] = adj[i].iter().map(|&j| u[j]).sum::<f64>() / k;
            next_v[i] = adj[i].iter().map(|&j| v[j]).sum::<f64>() / k;
        }
        u = next_u;
        v = next_v;
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

fn valid_edge(a: i64, b: i64, number_of_points: usize) -> Option<(usize, usize)> {
    if a >= 0 && b >= 0 {
        let a = a as usize;
        let b = b as usize;
        if a < number_of_points && b < number_of_points {
            return Some((a, b));
        }
    }
    None
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
