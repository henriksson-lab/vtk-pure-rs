use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;
use std::collections::{HashMap, VecDeque};

/// Split edges at sharp features while keeping the mesh watertight.
///
/// Duplicates vertices along sharp edges (dihedral angle > threshold)
/// so that each side of the edge has its own vertex copy. This enables
/// flat shading across sharp edges while keeping smooth shading elsewhere.
pub fn split_sharp_edges(input: &PolyData, angle_threshold_deg: f64) -> PolyData {
    if input.polys.is_empty() {
        return input.clone();
    }

    let cos_thresh = angle_threshold_deg.to_radians().cos();

    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let normals: Vec<[f64; 3]> = cells
        .iter()
        .map(|c| {
            if c.len() < 3 {
                return [0.0; 3];
            }
            let v0 = input.points.get(c[0] as usize);
            let v1 = input.points.get(c[1] as usize);
            let v2 = input.points.get(c[2] as usize);
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if l > 1e-15 {
                [n[0] / l, n[1] / l, n[2] / l]
            } else {
                [0.0; 3]
            }
        })
        .collect();

    let mut point_cells: Vec<Vec<usize>> = vec![Vec::new(); input.points.len()];
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for &vid in c {
            if vid >= 0 && (vid as usize) < point_cells.len() {
                point_cells[vid as usize].push(fi);
            }
        }
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut out_points = Points::<f64>::new();
    let mut point_map = Vec::with_capacity(input.points.len());
    for i in 0..input.points.len() {
        out_points.push(input.points.get(i));
        point_map.push(i);
    }

    let mut replacements: HashMap<(usize, i64), i64> = HashMap::new();

    for (point_id, incident_cells) in point_cells.iter().enumerate() {
        if incident_cells.len() <= 1 {
            continue;
        }

        let mut visited: HashMap<usize, i16> = incident_cells.iter().map(|&fi| (fi, -1)).collect();
        let mut region = 0i16;

        for &seed in incident_cells {
            if visited[&seed] >= 0 {
                continue;
            }

            let mut queue = VecDeque::new();
            visited.insert(seed, region);
            queue.push_back(seed);

            while let Some(fi) = queue.pop_front() {
                for neighbor_point in adjacent_points(&cells[fi], point_id as i64) {
                    let edge = if (point_id as i64) < neighbor_point {
                        (point_id as i64, neighbor_point)
                    } else {
                        (neighbor_point, point_id as i64)
                    };
                    let Some(edge_cells) = edge_faces.get(&edge) else {
                        continue;
                    };
                    if edge_cells.len() != 2 {
                        continue;
                    }
                    let nei = if edge_cells[0] == fi {
                        edge_cells[1]
                    } else if edge_cells[1] == fi {
                        edge_cells[0]
                    } else {
                        continue;
                    };
                    if !visited.contains_key(&nei) || visited[&nei] >= 0 {
                        continue;
                    }

                    let na = normals[fi];
                    let nb = normals[nei];
                    let dot = na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2];
                    if dot > cos_thresh {
                        visited.insert(nei, region);
                        queue.push_back(nei);
                    }
                }
            }

            region += 1;
        }

        if region <= 1 {
            continue;
        }

        let mut region_points: HashMap<i16, i64> = HashMap::new();
        for &fi in incident_cells {
            let cell_region = visited[&fi];
            if cell_region == 0 {
                continue;
            }
            let new_id = *region_points.entry(cell_region).or_insert_with(|| {
                let idx = out_points.len() as i64;
                out_points.push(input.points.get(point_id));
                point_map.push(point_id);
                idx
            });
            replacements.insert((fi, point_id as i64), new_id);
        }
    }

    let mut out_polys = CellArray::new();
    for (fi, c) in cells.iter().enumerate() {
        let ids: Vec<i64> = c
            .iter()
            .map(|&vid| replacements.get(&(fi, vid)).copied().unwrap_or(vid))
            .collect();
        out_polys.push_cell(&ids);
    }

    let mut pd = input.clone();
    pd.points = out_points;
    pd.polys = out_polys;
    *pd.point_data_mut() = remap_point_data(input.point_data(), &point_map);
    pd
}

fn remap_point_data(attrs: &DataSetAttributes, point_map: &[usize]) -> DataSetAttributes {
    let active_scalars = attrs.scalars().map(|a| a.name().to_string());
    let active_vectors = attrs.vectors().map(|a| a.name().to_string());
    let active_normals = attrs.normals().map(|a| a.name().to_string());
    let active_tcoords = attrs.tcoords().map(|a| a.name().to_string());
    let active_tensors = attrs.tensors().map(|a| a.name().to_string());
    let active_global_ids = attrs.global_ids().map(|a| a.name().to_string());
    let active_pedigree_ids = attrs.pedigree_ids().map(|a| a.name().to_string());
    let active_edge_flags = attrs.edge_flags().map(|a| a.name().to_string());
    let active_tangents = attrs.tangents().map(|a| a.name().to_string());
    let active_rational_weights = attrs.rational_weights().map(|a| a.name().to_string());
    let active_higher_order_degrees = attrs.higher_order_degrees().map(|a| a.name().to_string());
    let active_process_ids = attrs.process_ids().map(|a| a.name().to_string());

    let mut remapped = DataSetAttributes::new();
    for array in attrs.iter() {
        remapped.add_array(remap_array(array, point_map));
    }

    if let Some(name) = active_scalars {
        remapped.set_active_scalars(&name);
    }
    if let Some(name) = active_vectors {
        remapped.set_active_vectors(&name);
    }
    if let Some(name) = active_normals {
        remapped.set_active_normals(&name);
    }
    if let Some(name) = active_tcoords {
        remapped.set_active_tcoords(&name);
    }
    if let Some(name) = active_tensors {
        remapped.set_active_tensors(&name);
    }
    if let Some(name) = active_global_ids {
        remapped.set_active_global_ids(&name);
    }
    if let Some(name) = active_pedigree_ids {
        remapped.set_active_pedigree_ids(&name);
    }
    if let Some(name) = active_edge_flags {
        remapped.set_active_edge_flags(&name);
    }
    if let Some(name) = active_tangents {
        remapped.set_active_tangents(&name);
    }
    if let Some(name) = active_rational_weights {
        remapped.set_active_rational_weights(&name);
    }
    if let Some(name) = active_higher_order_degrees {
        remapped.set_active_higher_order_degrees(&name);
    }
    if let Some(name) = active_process_ids {
        remapped.set_active_process_ids(&name);
    }

    remapped
}

fn remap_array(array: &AnyDataArray, point_map: &[usize]) -> AnyDataArray {
    macro_rules! remap {
        ($array:expr, $variant:ident) => {{
            if $array.num_tuples() <= point_map.iter().copied().max().unwrap_or(0) {
                return AnyDataArray::$variant($array.clone());
            }
            AnyDataArray::$variant(remap_typed_array($array, point_map))
        }};
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

fn remap_typed_array<T: Scalar>(array: &DataArray<T>, point_map: &[usize]) -> DataArray<T> {
    let mut out = DataArray::new(array.name(), array.num_components());
    for &old_id in point_map {
        out.push_tuple(array.tuple(old_id));
    }
    out
}

fn adjacent_points(cell: &[i64], point_id: i64) -> Vec<i64> {
    let Some(spot) = cell.iter().position(|&vid| vid == point_id) else {
        return Vec::new();
    };
    let n = cell.len();
    if n < 2 {
        return Vec::new();
    }
    vec![cell[(spot + n - 1) % n], cell[(spot + 1) % n]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_90_degree() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = split_sharp_edges(&pd, 45.0);
        assert!(result.points.len() > 4); // duplicated
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn duplicates_point_data_with_split_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "id",
                vec![0.0, 1.0, 2.0, 3.0],
                1,
            )));
        pd.point_data_mut().set_active_scalars("id");

        let result = split_sharp_edges(&pd, 45.0);
        let arr = result.point_data().get_array("id").unwrap();

        assert_eq!(arr.num_tuples(), result.points.len());
        assert!(result.point_data().scalars().is_some());
    }

    #[test]
    fn flat_no_split() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = split_sharp_edges(&pd, 10.0);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = split_sharp_edges(&pd, 30.0);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn no_polys_passthrough() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);

        let result = split_sharp_edges(&pd, 30.0);
        assert_eq!(result, pd);
    }
}
