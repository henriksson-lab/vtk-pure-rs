//! Orient face normals consistently using propagation from a seed face.
use crate::data::{CellArray, PolyData};

pub fn orient_normals(mesh: &PolyData) -> PolyData {
    let tris: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nt = tris.len();
    if nt == 0 {
        return mesh.clone();
    }
    // Build edge-to-face adjacency
    let mut edge_faces: std::collections::HashMap<(i64, i64), Vec<usize>> =
        std::collections::HashMap::new();
    for (fi, tri) in tris.iter().enumerate() {
        let nc = tri.len();
        for i in 0..nc {
            let a = tri[i];
            let b = tri[(i + 1) % nc];
            let e = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(e).or_default().push(fi);
        }
    }
    let mut oriented = tris.clone();
    let mut visited = vec![false; nt];
    let mut queue = std::collections::VecDeque::new();
    visited[0] = true;
    queue.push_back(0);
    while let Some(fi) = queue.pop_front() {
        let nc = oriented[fi].len();
        for i in 0..nc {
            let a = oriented[fi][i];
            let b = oriented[fi][(i + 1) % nc];
            let e = if a < b { (a, b) } else { (b, a) };
            if let Some(neighbors) = edge_faces.get(&e) {
                for &fj in neighbors {
                    if visited[fj] {
                        continue;
                    }
                    visited[fj] = true;
                    // Check if fj has edge (b,a) or (a,b) - should be opposite direction
                    let ncj = oriented[fj].len();
                    let has_same_dir =
                        (0..ncj).any(|k| oriented[fj][k] == a && oriented[fj][(k + 1) % ncj] == b);
                    if has_same_dir {
                        oriented[fj].reverse(); // flip
                    }
                    queue.push_back(fj);
                }
            }
        }
    }
    let mut polys = CellArray::new();
    for tri in &oriented {
        polys.push_cell(tri);
    }
    let mut result = PolyData::new();
    result.points = mesh.points.clone();
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_orient() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [2, 1, 3]], // second face has inconsistent winding
        );
        let r = orient_normals(&mesh);
        assert_eq!(r.polys.num_cells(), 2);
    }
}
