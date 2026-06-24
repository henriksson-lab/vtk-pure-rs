//! Select connected region containing a seed vertex.

use crate::data::{CellArray, Points, PolyData};

/// Select the connected component containing the given seed vertex.
pub fn select_connected_region(mesh: &PolyData, seed: usize) -> PolyData {
    let n = mesh.points.len();
    if seed >= n {
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
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(seed);
    visited[seed] = true;
    while let Some(v) = queue.pop_front() {
        for &u in &nb[v] {
            if !visited[u] {
                visited[u] = true;
                queue.push_back(u);
            }
        }
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
    let mut pt_map = vec![0usize; n];
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for cell in &kept {
        polys.push_cell(
            &cell
                .iter()
                .map(|&v| pt_map[v as usize] as i64)
                .collect::<Vec<_>>(),
        );
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}

/// Count connected components.
pub fn count_connected_components(mesh: &PolyData) -> usize {
    let n = mesh.points.len();
    let mut parent: Vec<usize> = (0..n).collect();
    for cell in mesh.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        let first = cell[0] as usize;
        for i in 1..cell.len() {
            union(&mut parent, first, cell[i] as usize);
        }
    }
    let mut roots = std::collections::HashSet::new();
    let used: std::collections::HashSet<usize> = mesh
        .polys
        .iter()
        .flat_map(|c| c.iter().map(|&v| v as usize))
        .collect();
    for &v in &used {
        roots.insert(find(&mut parent, v));
    }
    roots.len()
}

fn find(p: &mut [usize], mut i: usize) -> usize {
    while p[i] != i {
        p[i] = p[p[i]];
        i = p[i];
    }
    i
}
fn union(p: &mut [usize], a: usize, b: usize) {
    let ra = find(p, a);
    let rb = find(p, b);
    if ra != rb {
        p[rb] = ra;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_select() {
        let mesh = PolyData::from_triangles(
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
        let r = select_connected_region(&mesh, 0);
        assert_eq!(r.polys.num_cells(), 1);
    }
    #[test]
    fn test_count() {
        let mesh = PolyData::from_triangles(
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
        assert_eq!(count_connected_components(&mesh), 2);
    }
}
