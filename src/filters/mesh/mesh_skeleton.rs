//! Approximate medial axis / skeleton of a mesh using vertex erosion.
use crate::data::{CellArray, Points, PolyData};

pub fn skeleton(mesh: &PolyData, erosion_steps: usize) -> PolyData {
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
    // Find boundary vertices
    let mut edge_count: std::collections::HashMap<(usize, usize), u32> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let e = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(e).or_insert(0) += 1;
        }
    }
    let mut boundary = vec![false; n];
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            boundary[a] = true;
            boundary[b] = true;
        }
    }
    // Erode: iteratively remove boundary vertices that are not endpoints
    let mut active = vec![true; n];
    for _ in 0..erosion_steps {
        let mut to_remove = Vec::new();
        for i in 0..n {
            if !active[i] || !boundary[i] {
                continue;
            }
            let active_neighbors: usize = adj[i].iter().filter(|&&j| active[j]).count();
            if active_neighbors > 1 {
                to_remove.push(i);
            } // not an endpoint
        }
        if to_remove.is_empty() {
            break;
        }
        for &i in &to_remove {
            active[i] = false;
        }
        // Update boundary
        boundary = vec![false; n];
        for (&(a, b), _) in &edge_count {
            if active[a] && active[b] {
                let shared_faces = adj[a]
                    .iter()
                    .filter(|&&j| adj[b].contains(&j) && active[j])
                    .count();
                if shared_faces <= 1 {
                    boundary[a] = true;
                    boundary[b] = true;
                }
            }
        }
    }
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pt_map = vec![0usize; n];
    for i in 0..n {
        if active[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i).try_into().unwrap());
        }
    }
    for (&(a, b), _) in &edge_count {
        if active[a] && active[b] {
            lines.push_cell(&[pt_map[a] as i64, pt_map[b] as i64]);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_skeleton() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.5, 1.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4], [1, 4, 2], [3, 5, 6], [3, 6, 4]],
        );
        let r = skeleton(&mesh, 2);
        assert!(r.points.len() > 0);
    }
}
