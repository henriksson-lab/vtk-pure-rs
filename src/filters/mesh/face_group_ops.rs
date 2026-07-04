//! Operations on face groups: split, merge, relabel, filter by size.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

/// Split a mesh into separate PolyData objects per face group.
pub fn split_by_label(mesh: &PolyData, label_array: &str) -> Vec<(i64, PolyData)> {
    let arr = match mesh.cell_data().get_array(label_array) {
        Some(a) => a,
        None => return Vec::new(),
    };
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let poly_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    let mut buf = [0.0f64];
    let mut groups: std::collections::BTreeMap<i64, Vec<usize>> = std::collections::BTreeMap::new();
    for ci in 0..all_cells.len() {
        let tuple_id = if arr.num_tuples() >= mesh.total_cells() {
            ci + poly_offset
        } else {
            ci
        };
        if tuple_id < arr.num_tuples() {
            arr.tuple_as_f64(tuple_id, &mut buf);
            groups.entry(buf[0] as i64).or_default().push(ci);
        }
    }
    groups
        .into_iter()
        .map(|(label, cells)| {
            let mut pts = Points::<f64>::new();
            let mut polys = CellArray::new();
            let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
            let mut point_ids = Vec::new();
            for &ci in &cells {
                let cell = &all_cells[ci];
                let mut ids = Vec::new();
                for &pid in cell {
                    let old = pid as usize;
                    let idx = *pm.entry(old).or_insert_with(|| {
                        let i = pts.len();
                        pts.push(mesh.points.get(old));
                        point_ids.push(old);
                        i
                    });
                    ids.push(idx as i64);
                }
                polys.push_cell(&ids);
            }
            let mut r = PolyData::new();
            r.points = pts;
            r.polys = polys;
            copy_attribute_tuples(mesh.point_data(), r.point_data_mut(), &point_ids);
            copy_poly_cell_attributes(mesh, r.cell_data_mut(), &cells);
            *r.field_data_mut() = mesh.field_data().clone();
            (label, r)
        })
        .collect()
}

/// Remove face groups with fewer than min_faces faces.
pub fn remove_small_groups(mesh: &PolyData, label_array: &str, min_faces: usize) -> PolyData {
    let arr = match mesh.cell_data().get_array(label_array) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let poly_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    let mut buf = [0.0f64];
    let mut counts: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    for ci in 0..mesh.polys.num_cells() {
        let tuple_id = if arr.num_tuples() >= mesh.total_cells() {
            ci + poly_offset
        } else {
            ci
        };
        if tuple_id < arr.num_tuples() {
            arr.tuple_as_f64(tuple_id, &mut buf);
            *counts.entry(buf[0] as i64).or_insert(0) += 1;
        }
    }

    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let keep: Vec<usize> = (0..all_cells.len())
        .filter(|&ci| {
            let tuple_id = if arr.num_tuples() >= mesh.total_cells() {
                ci + poly_offset
            } else {
                ci
            };
            if tuple_id < arr.num_tuples() {
                arr.tuple_as_f64(tuple_id, &mut buf);
                counts.get(&(buf[0] as i64)).copied().unwrap_or(0) >= min_faces
            } else {
                false
            }
        })
        .collect();

    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut point_ids = Vec::new();
    for &ci in &keep {
        let cell = &all_cells[ci];
        let mut ids = Vec::new();
        for &pid in cell {
            let old = pid as usize;
            let idx = *pm.entry(old).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(old));
                point_ids.push(old);
                i
            });
            ids.push(idx as i64);
        }
        polys.push_cell(&ids);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    copy_attribute_tuples(mesh.point_data(), result.point_data_mut(), &point_ids);
    copy_poly_cell_attributes(mesh, result.cell_data_mut(), &keep);
    *result.field_data_mut() = mesh.field_data().clone();
    result
}

/// Relabel groups sequentially starting from 0.
pub fn relabel_groups(mesh: &PolyData, label_array: &str) -> PolyData {
    let arr = match mesh.cell_data().get_array(label_array) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let poly_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    let mut buf = [0.0f64];
    let mut mapping: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    let mut next = 0;
    let mut data = vec![0.0; arr.num_tuples()];
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        data[i] = buf[0];
    }
    for i in 0..mesh.polys.num_cells() {
        let tuple_id = if arr.num_tuples() >= mesh.total_cells() {
            i + poly_offset
        } else {
            i
        };
        if tuple_id < arr.num_tuples() {
            arr.tuple_as_f64(tuple_id, &mut buf);
            let old = buf[0] as i64;
            let new = *mapping.entry(old).or_insert_with(|| {
                let n = next;
                next += 1;
                n
            });
            data[tuple_id] = new as f64;
        }
    }

    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(label_array, data, 1)));
    result
}

fn copy_poly_cell_attributes(input: &PolyData, target: &mut DataSetAttributes, poly_ids: &[usize]) {
    let poly_offset = input.verts.num_cells() + input.lines.num_cells();
    for array in input.cell_data().iter() {
        let ids: Vec<usize> = if array.num_tuples() >= input.total_cells() {
            poly_ids.iter().map(|&id| id + poly_offset).collect()
        } else {
            poly_ids.to_vec()
        };
        if ids.iter().all(|&id| id < array.num_tuples()) {
            target.add_array(subset_array(array, &ids));
        }
    }
    copy_active_attributes(input.cell_data(), target);
}

fn copy_attribute_tuples(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    tuple_ids: &[usize],
) {
    for array in source.iter() {
        if tuple_ids.iter().all(|&id| id < array.num_tuples()) {
            target.add_array(subset_array(array, tuple_ids));
        }
    }
    copy_active_attributes(source, target);
}

fn copy_active_attributes(source: &DataSetAttributes, target: &mut DataSetAttributes) {
    if let Some(array) = source.scalars() {
        target.set_active_scalars(array.name());
    }
    if let Some(array) = source.vectors() {
        target.set_active_vectors(array.name());
    }
    if let Some(array) = source.normals() {
        target.set_active_normals(array.name());
    }
    if let Some(array) = source.tcoords() {
        target.set_active_tcoords(array.name());
    }
    if let Some(array) = source.tensors() {
        target.set_active_tensors(array.name());
    }
    if let Some(array) = source.global_ids() {
        target.set_active_global_ids(array.name());
    }
    if let Some(array) = source.pedigree_ids() {
        target.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = source.edge_flags() {
        target.set_active_edge_flags(array.name());
    }
    if let Some(array) = source.tangents() {
        target.set_active_tangents(array.name());
    }
    if let Some(array) = source.rational_weights() {
        target.set_active_rational_weights(array.name());
    }
    if let Some(array) = source.higher_order_degrees() {
        target.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = source.process_ids() {
        target.set_active_process_ids(array.name());
    }
}

fn subset_array(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! subset {
        ($arr:expr, $variant:ident) => {{
            let nc = $arr.num_components();
            let mut values = Vec::with_capacity(tuple_ids.len() * nc);
            for &tuple_id in tuple_ids {
                values.extend_from_slice($arr.tuple(tuple_id));
            }
            AnyDataArray::$variant(DataArray::from_vec($arr.name(), values, nc))
        }};
    }
    match array {
        AnyDataArray::F32(arr) => subset!(arr, F32),
        AnyDataArray::F64(arr) => subset!(arr, F64),
        AnyDataArray::I8(arr) => subset!(arr, I8),
        AnyDataArray::I16(arr) => subset!(arr, I16),
        AnyDataArray::I32(arr) => subset!(arr, I32),
        AnyDataArray::I64(arr) => subset!(arr, I64),
        AnyDataArray::U8(arr) => subset!(arr, U8),
        AnyDataArray::U16(arr) => subset!(arr, U16),
        AnyDataArray::U32(arr) => subset!(arr, U32),
        AnyDataArray::U64(arr) => subset!(arr, U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "g",
                vec![1.0, 2.0],
                1,
            )));
        let parts = split_by_label(&mesh, "g");
        assert_eq!(parts.len(), 2);
    }
    #[test]
    fn remove_small() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [4, 5, 6]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "g",
                vec![1.0, 1.0, 2.0],
                1,
            )));
        let result = remove_small_groups(&mesh, "g", 2);
        assert_eq!(result.polys.num_cells(), 2); // only group 1
    }
    #[test]
    fn relabel() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "g",
                vec![10.0, 20.0],
                1,
            )));
        let result = relabel_groups(&mesh, "g");
        let arr = result.cell_data().get_array("g").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
