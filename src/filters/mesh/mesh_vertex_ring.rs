//! Extract N-ring neighborhood of a vertex.
use crate::data::{CellArray, Points, PolyData};
pub fn extract_vertex_ring(mesh: &PolyData, vertex: usize, rings: usize) -> PolyData {
    let n = mesh.points.len();
    if vertex >= n {
        return PolyData::new();
    }
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
            }
        }
    }
    let mut visited = vec![false; n];
    visited[vertex] = true;
    let mut frontier = vec![vertex];
    for _ in 0..rings {
        let mut next = Vec::new();
        for &v in &frontier {
            for &u in &nb[v] {
                if !visited[u] {
                    visited[u] = true;
                    next.push(u);
                }
            }
        }
        frontier = next;
    }
    let mut used = vec![false; n];
    let mut kept = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.iter().all(|&v| visited[v as usize]) {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.to_vec());
        }
    }
    let mut pm = vec![0usize; n];
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        if used[i] {
            pm[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for c in &kept {
        polys.push_cell(&c.iter().map(|&v| pm[v as usize] as i64).collect::<Vec<_>>());
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ring() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2], [1, 4, 3]],
        );
        let r = extract_vertex_ring(&m, 0, 1);
        assert!(r.polys.num_cells() >= 1);
    }
}
