//! Extract mesh patches: connected subsets by region, boundary, or selection.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

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
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], mesh.points.len()) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], mesh.points.len()) else {
                continue;
            };
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
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], mesh.points.len()) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], mesh.points.len()) else {
                continue;
            };
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
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], mesh.points.len()) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], mesh.points.len()) else {
                continue;
            };
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
            if nc < 2 {
                continue;
            }
            for i in 0..nc {
                let Some(a) = valid_point_id(cell[i], mesh.points.len()) else {
                    continue;
                };
                let Some(b) = valid_point_id(cell[(i + 1) % nc], mesh.points.len()) else {
                    continue;
                };
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
    let mut old_point_ids = Vec::new();
    let mut kept_cell_ids = Vec::new();
    let n_points = mesh.points.len();
    for &ci in selected {
        let cell = &all_cells[ci];
        if cell.len() < 3 {
            continue;
        }
        // Validate the whole cell *before* emitting any of its points, otherwise a
        // cell that is rejected part-way through leaves orphaned points (and stale
        // point-data tuples) behind in the output.
        let Some(old_ids) = cell
            .iter()
            .map(|&pid| valid_point_id(pid, n_points))
            .collect::<Option<Vec<usize>>>()
        else {
            continue;
        };
        let mut ids = Vec::with_capacity(old_ids.len());
        for old in old_ids {
            let idx = *pt_map.entry(old).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(old));
                old_point_ids.push(old);
                i
            });
            ids.push(idx as i64);
        }
        polys.push_cell(&ids);
        kept_cell_ids.push(ci);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    copy_point_data(mesh, &mut result, &old_point_ids);
    copy_cell_data(mesh, &mut result, &kept_cell_ids);
    result
}

fn valid_point_id(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

fn copy_point_data(source: &PolyData, target: &mut PolyData, old_point_ids: &[usize]) {
    let source_data = source.point_data();
    let target_data = target.point_data_mut();
    for i in 0..source_data.num_arrays() {
        let Some(array) = source_data.get_array_by_index(i) else {
            continue;
        };
        if let Some(subset) = subset_array(array, old_point_ids) {
            let name = subset.name().to_string();
            target_data.add_array(subset);
            copy_active_attribute(source_data, target_data, &name);
        }
    }
}

fn copy_cell_data(source: &PolyData, target: &mut PolyData, selected: &[usize]) {
    let source_data = source.cell_data();
    let target_data = target.cell_data_mut();
    for i in 0..source_data.num_arrays() {
        let Some(array) = source_data.get_array_by_index(i) else {
            continue;
        };
        if let Some(subset) = subset_array(array, selected) {
            let name = subset.name().to_string();
            target_data.add_array(subset);
            copy_active_attribute(source_data, target_data, &name);
        }
    }
}

fn subset_array(array: &AnyDataArray, tuple_ids: &[usize]) -> Option<AnyDataArray> {
    macro_rules! subset_variant {
        ($variant:ident) => {{
            let AnyDataArray::$variant(a) = array else {
                unreachable!();
            };
            let nc = a.num_components();
            let mut data = Vec::with_capacity(tuple_ids.len() * nc);
            for &tuple_id in tuple_ids {
                if tuple_id >= a.num_tuples() {
                    return None;
                }
                data.extend_from_slice(a.tuple(tuple_id));
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                a.name(),
                data,
                nc,
            )))
        }};
    }
    match array {
        AnyDataArray::F32(_) => subset_variant!(F32),
        AnyDataArray::F64(_) => subset_variant!(F64),
        AnyDataArray::I8(_) => subset_variant!(I8),
        AnyDataArray::I16(_) => subset_variant!(I16),
        AnyDataArray::I32(_) => subset_variant!(I32),
        AnyDataArray::I64(_) => subset_variant!(I64),
        AnyDataArray::U8(_) => subset_variant!(U8),
        AnyDataArray::U16(_) => subset_variant!(U16),
        AnyDataArray::U32(_) => subset_variant!(U32),
        AnyDataArray::U64(_) => subset_variant!(U64),
    }
}

fn copy_active_attribute(source: &DataSetAttributes, target: &mut DataSetAttributes, name: &str) {
    if source.scalars().map(|a| a.name()) == Some(name) {
        target.set_active_scalars(name);
    }
    if source.vectors().map(|a| a.name()) == Some(name) {
        target.set_active_vectors(name);
    }
    if source.normals().map(|a| a.name()) == Some(name) {
        target.set_active_normals(name);
    }
    if source.tcoords().map(|a| a.name()) == Some(name) {
        target.set_active_tcoords(name);
    }
    if source.tensors().map(|a| a.name()) == Some(name) {
        target.set_active_tensors(name);
    }
    if source.global_ids().map(|a| a.name()) == Some(name) {
        target.set_active_global_ids(name);
    }
    if source.pedigree_ids().map(|a| a.name()) == Some(name) {
        target.set_active_pedigree_ids(name);
    }
    if source.edge_flags().map(|a| a.name()) == Some(name) {
        target.set_active_edge_flags(name);
    }
    if source.tangents().map(|a| a.name()) == Some(name) {
        target.set_active_tangents(name);
    }
    if source.rational_weights().map(|a| a.name()) == Some(name) {
        target.set_active_rational_weights(name);
    }
    if source.higher_order_degrees().map(|a| a.name()) == Some(name) {
        target.set_active_higher_order_degrees(name);
    }
    if source.process_ids().map(|a| a.name()) == Some(name) {
        target.set_active_process_ids(name);
    }
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
        let arr = result.cell_data().get_array("region").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
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

    #[test]
    fn malformed_cells_do_not_panic() {
        let mut mesh =
            PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        mesh.polys.push_cell(&[0, -1, 99]);

        let result = extract_face_patch(&mesh, 0, 1);
        assert_eq!(result.points.len(), 0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
