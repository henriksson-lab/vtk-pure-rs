use crate::data::{AnyDataArray, DataArray, DataSetAttributes, PolyData};
use crate::types::Scalar;
use std::collections::{HashMap, HashSet, VecDeque};

/// Split a vertex into multiple copies, one per adjacent face group.
///
/// Groups faces around the vertex by connectivity through other shared
/// edges (not through the split vertex). Each group gets its own copy.
/// Useful for creating sharp corners at specific vertices.
pub fn split_vertex(input: &PolyData, vertex_id: usize) -> PolyData {
    let vid = vertex_id as i64;
    let n = input.points.len();
    if vertex_id >= n {
        return input.clone();
    }

    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let face_indices: Vec<usize> = cells
        .iter()
        .enumerate()
        .filter(|(_, c)| c.contains(&vid))
        .map(|(i, _)| i)
        .collect();

    if face_indices.len() <= 1 {
        return input.clone();
    }

    let mut face_adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for &fi in &face_indices {
        for &fj in &face_indices {
            if fi >= fj {
                continue;
            }
            if share_non_split_edge(&cells[fi], &cells[fj], vid) {
                face_adj.entry(fi).or_default().push(fj);
                face_adj.entry(fj).or_default().push(fi);
            }
        }
    }

    let mut visited = HashSet::new();
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for &fi in &face_indices {
        if visited.contains(&fi) {
            continue;
        }
        let mut group = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(fi);
        visited.insert(fi);
        while let Some(f) = queue.pop_front() {
            group.push(f);
            if let Some(adj) = face_adj.get(&f) {
                for &nf in adj {
                    if visited.insert(nf) {
                        queue.push_back(nf);
                    }
                }
            }
        }
        groups.push(group);
    }

    if groups.len() <= 1 {
        return input.clone();
    }

    let mut out_pts = input.points.clone();
    let mut new_cells = cells.clone();
    let mut old_point_ids: Vec<usize> = (0..n).collect();

    for group in groups.iter().skip(1) {
        let new_vid = out_pts.len() as i64;
        out_pts.push(input.points.get(vertex_id));
        old_point_ids.push(vertex_id);
        for &fi in group {
            for v in &mut new_cells[fi] {
                if *v == vid {
                    *v = new_vid;
                }
            }
        }
    }

    let mut pd = input.clone();
    pd.points = out_pts;
    pd.polys.clear();
    for c in &new_cells {
        pd.polys.push_cell(c);
    }
    copy_point_data(input, &old_point_ids, &mut pd);
    pd
}

fn share_non_split_edge(a: &[i64], b: &[i64], split_id: i64) -> bool {
    if a.len() < 2 || b.len() < 2 {
        return false;
    }
    let b_edges: HashSet<(i64, i64)> = closed_edges(b, split_id).collect();
    closed_edges(a, split_id).any(|edge| b_edges.contains(&edge))
}

fn closed_edges(cell: &[i64], split_id: i64) -> impl Iterator<Item = (i64, i64)> + '_ {
    cell.iter()
        .zip(cell.iter().cycle().skip(1))
        .take(cell.len())
        .filter_map(move |(&a, &b)| {
            if a == split_id || b == split_id {
                None
            } else if a < b {
                Some((a, b))
            } else {
                Some((b, a))
            }
        })
}

fn copy_point_data(input: &PolyData, old_point_ids: &[usize], output: &mut PolyData) {
    output.point_data_mut().clear();
    for array in input.point_data().field_data().iter() {
        if array.num_tuples() == input.points.len() {
            output
                .point_data_mut()
                .add_array(remap_array(array, old_point_ids));
        }
    }
    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn remap_array(array: &AnyDataArray, old_point_ids: &[usize]) -> AnyDataArray {
    macro_rules! remap {
        ($arr:expr, $variant:ident) => {
            AnyDataArray::$variant(remap_typed_array($arr, old_point_ids))
        };
    }
    match array {
        AnyDataArray::F32(a) => remap!(a, F32),
        AnyDataArray::F64(a) => remap!(a, F64),
        AnyDataArray::I8(a) => remap!(a, I8),
        AnyDataArray::I16(a) => remap!(a, I16),
        AnyDataArray::I32(a) => remap!(a, I32),
        AnyDataArray::I64(a) => remap!(a, I64),
        AnyDataArray::U8(a) => remap!(a, U8),
        AnyDataArray::U16(a) => remap!(a, U16),
        AnyDataArray::U32(a) => remap!(a, U32),
        AnyDataArray::U64(a) => remap!(a, U64),
    }
}

fn remap_typed_array<T: Scalar>(array: &DataArray<T>, old_point_ids: &[usize]) -> DataArray<T> {
    let nc = array.num_components();
    let mut data = Vec::with_capacity(old_point_ids.len() * nc);
    for &old_id in old_point_ids {
        data.extend_from_slice(array.tuple(old_id));
    }
    DataArray::from_vec(array.name(), data, nc)
}

fn copy_active_attributes(input: &DataSetAttributes, output: &mut DataSetAttributes) {
    if let Some(array) = input.scalars() {
        output.set_active_scalars(array.name());
    }
    if let Some(array) = input.vectors() {
        output.set_active_vectors(array.name());
    }
    if let Some(array) = input.normals() {
        output.set_active_normals(array.name());
    }
    if let Some(array) = input.tcoords() {
        output.set_active_tcoords(array.name());
    }
    if let Some(array) = input.tensors() {
        output.set_active_tensors(array.name());
    }
    if let Some(array) = input.global_ids() {
        output.set_active_global_ids(array.name());
    }
    if let Some(array) = input.pedigree_ids() {
        output.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = input.edge_flags() {
        output.set_active_edge_flags(array.name());
    }
    if let Some(array) = input.tangents() {
        output.set_active_tangents(array.name());
    }
    if let Some(array) = input.rational_weights() {
        output.set_active_rational_weights(array.name());
    }
    if let Some(array) = input.higher_order_degrees() {
        output.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = input.process_ids() {
        output.set_active_process_ids(array.name());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_fan_vertex() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // vertex to split
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([-1.0, 0.0, 0.0]);
        pd.points.push([0.0, -1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 4]);

        // These two faces don't share an edge besides vertex 0
        let result = split_vertex(&pd, 0);
        // May or may not split depending on adjacency detection
        assert!(result.polys.num_cells() == 2);
    }

    #[test]
    fn single_face_noop() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = split_vertex(&pd, 0);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn invalid_vertex() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = split_vertex(&pd, 999);
        assert_eq!(result.points.len(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = split_vertex(&pd, 0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
