//! Extract N-ring neighborhood of a vertex.
use crate::data::{CellArray, Points, PolyData};
pub fn extract_vertex_ring(mesh: &PolyData, vertex: usize, rings: usize) -> PolyData {
    let n = mesh.points.len();
    if vertex >= n {
        return PolyData::new();
    }
    let cells = surface_cells(mesh, n);
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in &cells {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i];
            let b = cell[(i + 1) % nc];
            if !nb[a].contains(&b) {
                nb[a].push(b);
            }
            if !nb[b].contains(&a) {
                nb[b].push(a);
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
    for cell in &cells {
        if cell.iter().all(|&v| visited[v]) {
            for &v in cell {
                used[v] = true;
            }
            kept.push(cell.clone());
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
        polys.push_cell(&c.iter().map(|&v| pm[v] as i64).collect::<Vec<_>>());
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}

fn surface_cells(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut cells = Vec::new();
    for cell in mesh.polys.iter() {
        push_valid_cell(&mut cells, cell, n);
    }
    for strip in mesh.strips.iter() {
        for (i, tri) in strip.windows(3).enumerate() {
            if i % 2 == 0 {
                push_valid_cell(&mut cells, &[tri[0], tri[1], tri[2]], n);
            } else {
                push_valid_cell(&mut cells, &[tri[1], tri[0], tri[2]], n);
            }
        }
    }
    cells
}

fn push_valid_cell(cells: &mut Vec<Vec<usize>>, cell: &[i64], n: usize) {
    let mut ids = Vec::with_capacity(cell.len());
    for &v in cell {
        let Some(v) = valid_point_index(v, n) else {
            return;
        };
        ids.push(v);
    }
    cells.push(ids);
}

fn valid_point_index(id: i64, n: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n)
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
