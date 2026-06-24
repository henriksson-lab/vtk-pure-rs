//! Build face-face adjacency (dual graph edges).
use crate::data::PolyData;
pub struct FaceAdjacency {
    pub adjacency: Vec<Vec<usize>>,
    pub num_faces: usize,
}
pub fn face_adjacency(mesh: &PolyData) -> FaceAdjacency {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in cells.iter().enumerate() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); nc];
    for (_, faces) in &ef {
        for i in 0..faces.len() {
            for j in i + 1..faces.len() {
                adj[faces[i]].push(faces[j]);
                adj[faces[j]].push(faces[i]);
            }
        }
    }
    FaceAdjacency {
        adjacency: adj,
        num_faces: nc,
    }
}
pub fn face_adjacency_count(mesh: &PolyData) -> Vec<usize> {
    let fa = face_adjacency(mesh);
    fa.adjacency.iter().map(|a| a.len()).collect()
}
pub fn is_face_connected(mesh: &PolyData) -> bool {
    let fa = face_adjacency(mesh);
    if fa.num_faces == 0 {
        return true;
    }
    let mut visited = vec![false; fa.num_faces];
    let mut queue = std::collections::VecDeque::new();
    visited[0] = true;
    queue.push_back(0);
    while let Some(ci) = queue.pop_front() {
        for &ni in &fa.adjacency[ci] {
            if !visited[ni] {
                visited[ni] = true;
                queue.push_back(ni);
            }
        }
    }
    visited.iter().all(|&v| v)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_adj() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let fa = face_adjacency(&m);
        assert_eq!(fa.adjacency[0].len(), 1);
        assert_eq!(fa.adjacency[1].len(), 1);
    }
    #[test]
    fn test_connected() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        assert!(is_face_connected(&m));
    }
    #[test]
    fn test_disconnected() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.5, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        assert!(!is_face_connected(&m));
    }
}
