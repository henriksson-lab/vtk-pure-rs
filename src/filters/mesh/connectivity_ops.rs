//! Connectivity operations: extract components, check connectivity, bridge gaps.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Label each face by its connected component ID.
pub fn label_connected_components(mesh: &PolyData) -> PolyData {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let n = all_cells.len();
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

    let mut labels = vec![usize::MAX; n];
    let mut next = 0;
    for seed in 0..n {
        if labels[seed] != usize::MAX {
            continue;
        }
        let mut q = std::collections::VecDeque::new();
        q.push_back(seed);
        labels[seed] = next;
        while let Some(ci) = q.pop_front() {
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
        next += 1;
    }

    let data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ComponentId",
            data,
            1,
        )));
    result
}

/// Count connected components.
pub fn num_connected_components(mesh: &PolyData) -> usize {
    let labeled = label_connected_components(mesh);
    match labeled.cell_data().get_array("ComponentId") {
        Some(arr) => {
            let mut max_id = -1i64;
            let mut buf = [0.0f64];
            for i in 0..arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                max_id = max_id.max(buf[0] as i64);
            }
            if max_id >= 0 {
                (max_id + 1) as usize
            } else {
                0
            }
        }
        None => 0,
    }
}

/// Check if a mesh is fully connected (single component).
pub fn is_connected(mesh: &PolyData) -> bool {
    num_connected_components(mesh) <= 1
}

/// Extract component by ID.
pub fn extract_component(mesh: &PolyData, component_id: usize) -> PolyData {
    let labeled = label_connected_components(mesh);
    let arr = match labeled.cell_data().get_array("ComponentId") {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut buf = [0.0f64];
    let selected: Vec<usize> = (0..all_cells.len())
        .filter(|&ci| {
            if ci < arr.num_tuples() {
                arr.tuple_as_f64(ci, &mut buf);
                buf[0] as usize == component_id
            } else {
                false
            }
        })
        .collect();

    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for &ci in &selected {
        let cell = &all_cells[ci];
        let mut ids = Vec::new();
        for &pid in cell {
            let old = pid as usize;
            let idx = *pm.entry(old).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(old));
                i
            });
            ids.push(idx as i64);
        }
        polys.push_cell(&ids);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}

/// Get sizes (face counts) of each component, sorted descending.
pub fn component_sizes(mesh: &PolyData) -> Vec<usize> {
    let labeled = label_connected_components(mesh);
    let arr = match labeled.cell_data().get_array("ComponentId") {
        Some(a) => a,
        None => return Vec::new(),
    };
    let mut counts: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut buf = [0.0f64];
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        *counts.entry(buf[0] as usize).or_insert(0) += 1;
    }
    let mut sizes: Vec<usize> = counts.values().cloned().collect();
    sizes.sort_by(|a, b| b.cmp(a));
    sizes
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn single_component() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        assert!(is_connected(&mesh));
        assert_eq!(num_connected_components(&mesh), 1);
    }
    #[test]
    fn two_components() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [5.0, 0.0, 0.0],
                [6.0, 0.0, 0.0],
                [5.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        assert!(!is_connected(&mesh));
        assert_eq!(num_connected_components(&mesh), 2);
    }
    #[test]
    fn extract() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [5.0, 0.0, 0.0],
                [6.0, 0.0, 0.0],
                [5.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let comp = extract_component(&mesh, 0);
        assert_eq!(comp.polys.num_cells(), 1);
    }
    #[test]
    fn sizes() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
                [5.0, 0.0, 0.0],
                [6.0, 0.0, 0.0],
                [5.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [4, 5, 6]],
        );
        let s = component_sizes(&mesh);
        assert_eq!(s[0], 2); // largest first
        assert_eq!(s[1], 1);
    }
}
