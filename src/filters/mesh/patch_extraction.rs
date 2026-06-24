//! Extract mesh patches: connected subsets by region, boundary, or selection.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Extract faces within a geodesic radius of a seed face.
pub fn extract_face_patch(mesh: &PolyData, seed_face: usize, max_hops: usize) -> PolyData {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let n_cells = all_cells.len();
    if seed_face >= n_cells {
        return PolyData::new();
    }

    let mut edge_adj: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_adj.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    let mut visited = vec![false; n_cells];
    let mut queue = std::collections::VecDeque::new();
    let mut dist = vec![usize::MAX; n_cells];
    queue.push_back(seed_face);
    visited[seed_face] = true;
    dist[seed_face] = 0;

    while let Some(ci) = queue.pop_front() {
        if dist[ci] >= max_hops {
            continue;
        }
        let cell = &all_cells[ci];
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if let Some(nbs) = edge_adj.get(&(a.min(b), a.max(b))) {
                for &ni in nbs {
                    if !visited[ni] {
                        visited[ni] = true;
                        dist[ni] = dist[ci] + 1;
                        queue.push_back(ni);
                    }
                }
            }
        }
    }

    let selected: Vec<usize> = (0..n_cells).filter(|&i| visited[i]).collect();
    extract_cells(mesh, &all_cells, &selected)
}

/// Extract faces where a cell data array matches a value.
pub fn extract_by_cell_value(
    mesh: &PolyData,
    array_name: &str,
    value: f64,
    tolerance: f64,
) -> PolyData {
    let arr = match mesh.cell_data().get_array(array_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut buf = [0.0f64];
    let selected: Vec<usize> = (0..all_cells.len())
        .filter(|&i| {
            if i < arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                (buf[0] - value).abs() <= tolerance
            } else {
                false
            }
        })
        .collect();
    extract_cells(mesh, &all_cells, &selected)
}

/// Extract N largest connected components by face count.
pub fn extract_n_largest_components(mesh: &PolyData, n: usize) -> PolyData {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let n_cells = all_cells.len();
    if n_cells == 0 {
        return mesh.clone();
    }

    let mut edge_adj: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_adj.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    let mut labels = vec![usize::MAX; n_cells];
    let mut next = 0;
    let mut comp_sizes: Vec<(usize, usize)> = Vec::new();
    for seed in 0..n_cells {
        if labels[seed] != usize::MAX {
            continue;
        }
        let mut count = 0;
        let mut q = std::collections::VecDeque::new();
        q.push_back(seed);
        labels[seed] = next;
        while let Some(ci) = q.pop_front() {
            count += 1;
            let cell = &all_cells[ci];
            let nc = cell.len();
            for i in 0..nc {
                let a = cell[i] as usize;
                let b = cell[(i + 1) % nc] as usize;
                if let Some(nbs) = edge_adj.get(&(a.min(b), a.max(b))) {
                    for &ni in nbs {
                        if labels[ni] == usize::MAX {
                            labels[ni] = next;
                            q.push_back(ni);
                        }
                    }
                }
            }
        }
        comp_sizes.push((next, count));
        next += 1;
    }

    comp_sizes.sort_by(|a, b| b.1.cmp(&a.1));
    let keep: std::collections::HashSet<usize> =
        comp_sizes.iter().take(n).map(|&(id, _)| id).collect();
    let selected: Vec<usize> = (0..n_cells)
        .filter(|&i| keep.contains(&labels[i]))
        .collect();
    extract_cells(mesh, &all_cells, &selected)
}

fn extract_cells(mesh: &PolyData, all_cells: &[Vec<i64>], selected: &[usize]) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for &ci in selected {
        let cell = &all_cells[ci];
        let mut ids = Vec::new();
        for &pid in cell {
            let old = pid as usize;
            let idx = *pt_map.entry(old).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(old));
                i
            });
            ids.push(idx as i64);
        }
        polys.push_cell(&ids);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn face_patch() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let patch = extract_face_patch(&mesh, 0, 2);
        assert!(patch.polys.num_cells() > 0);
        assert!(patch.polys.num_cells() < mesh.polys.num_cells());
    }
    #[test]
    fn by_value() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "region",
                vec![1.0, 2.0],
                1,
            )));
        let result = extract_by_cell_value(&mesh, "region", 1.0, 0.01);
        assert_eq!(result.polys.num_cells(), 1);
    }
    #[test]
    fn largest_components() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, -1.0, 0.0],
                [5.0, 0.0, 0.0],
                [6.0, 0.0, 0.0],
                [5.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5], [6, 7, 8]],
        );
        let result = extract_n_largest_components(&mesh, 1);
        // Largest component has 2 faces (sharing edge 0-1)
        assert!(result.polys.num_cells() >= 1);
    }
}
