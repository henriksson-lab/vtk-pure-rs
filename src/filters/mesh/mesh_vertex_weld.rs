//! Weld vertices closer than tolerance.
use std::collections::HashMap;

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
pub fn weld_vertices(mesh: &PolyData, tolerance: f64) -> PolyData {
    let n: usize = mesh.points.len();
    let tolerance = if tolerance.is_finite() && tolerance > 0.0 {
        tolerance
    } else {
        0.0
    };
    let tol2: f64 = tolerance * tolerance;
    let inv_tol: f64 = if tolerance > 0.0 {
        1.0 / tolerance
    } else {
        1.0
    };

    let mut new_points: Points<f64> = Points::new();
    let mut point_map: Vec<usize> = vec![0; n];
    let mut representative_points: Vec<usize> = Vec::new();
    let mut grid: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();

    for i in 0..n {
        let p = mesh.points.get(i);
        let key = (
            (p[0] * inv_tol).round() as i64,
            (p[1] * inv_tol).round() as i64,
            (p[2] * inv_tol).round() as i64,
        );

        let mut found: Option<usize> = None;
        'search: for dx in -1i64..=1 {
            for dy in -1i64..=1 {
                for dz in -1i64..=1 {
                    let Some(nkey) = offset_key(key, dx, dy, dz) else {
                        continue;
                    };
                    if let Some(indices) = grid.get(&nkey) {
                        for &idx in indices {
                            let q = new_points.get(idx);
                            let d2: f64 = (p[0] - q[0]) * (p[0] - q[0])
                                + (p[1] - q[1]) * (p[1] - q[1])
                                + (p[2] - q[2]) * (p[2] - q[2]);
                            if d2 <= tol2 {
                                found = Some(idx);
                                break 'search;
                            }
                        }
                    }
                }
            }
        }

        match found {
            Some(idx) => {
                point_map[i] = idx;
            }
            None => {
                let new_idx: usize = new_points.len();
                new_points.push(p);
                point_map[i] = new_idx;
                representative_points.push(i);
                grid.entry(key).or_default().push(new_idx);
            }
        }
    }

    let mut r = mesh.clone();
    r.points = new_points;
    let mut kept_cell_ids = Vec::new();
    let mut cell_offset = 0usize;
    r.verts = remap_cell_array(
        &mesh.verts,
        &point_map,
        1,
        &mut cell_offset,
        &mut kept_cell_ids,
    );
    r.lines = remap_cell_array(
        &mesh.lines,
        &point_map,
        2,
        &mut cell_offset,
        &mut kept_cell_ids,
    );
    r.polys = remap_cell_array(
        &mesh.polys,
        &point_map,
        3,
        &mut cell_offset,
        &mut kept_cell_ids,
    );
    r.strips = remap_cell_array(
        &mesh.strips,
        &point_map,
        3,
        &mut cell_offset,
        &mut kept_cell_ids,
    );
    *r.point_data_mut() = remap_attribute_data(mesh.point_data(), &representative_points, n);
    *r.cell_data_mut() = remap_attribute_data(mesh.cell_data(), &kept_cell_ids, mesh.total_cells());
    r
}

fn offset_key(key: (i64, i64, i64), dx: i64, dy: i64, dz: i64) -> Option<(i64, i64, i64)> {
    Some((
        key.0.checked_add(dx)?,
        key.1.checked_add(dy)?,
        key.2.checked_add(dz)?,
    ))
}

fn remap_cell_array(
    cells: &CellArray,
    point_map: &[usize],
    min_unique: usize,
    cell_offset: &mut usize,
    kept_cell_ids: &mut Vec<usize>,
) -> CellArray {
    let mut out = CellArray::new();
    for (local_cell_id, cell) in cells.iter().enumerate() {
        let input_cell_id = *cell_offset + local_cell_id;
        if !cell.iter().all(|&id| valid_point_id(id, point_map.len())) {
            continue;
        }

        let mapped: Vec<i64> = cell
            .iter()
            .map(|&id| point_map[id as usize] as i64)
            .collect();
        let mut unique = mapped.clone();
        unique.sort();
        unique.dedup();
        if unique.len() >= min_unique {
            out.push_cell(&mapped);
            kept_cell_ids.push(input_cell_id);
        }
    }
    *cell_offset += cells.num_cells();
    out
}

fn valid_point_id(id: i64, number_of_points: usize) -> bool {
    id >= 0 && (id as usize) < number_of_points
}

fn remap_attribute_data(
    input: &DataSetAttributes,
    tuple_ids: &[usize],
    number_of_input_tuples: usize,
) -> DataSetAttributes {
    let mut output = DataSetAttributes::new();

    for array in input.iter() {
        if let Some(remapped) = remap_array_tuples(array, tuple_ids, number_of_input_tuples) {
            output.add_array(remapped);
        }
    }

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

    output
}

fn remap_array_tuples(
    array: &AnyDataArray,
    tuple_ids: &[usize],
    number_of_input_tuples: usize,
) -> Option<AnyDataArray> {
    if array.num_tuples() != number_of_input_tuples {
        return None;
    }

    macro_rules! remap {
        ($array:expr, $variant:ident) => {{
            let number_of_components = $array.num_components();
            let mut values = Vec::with_capacity(tuple_ids.len() * number_of_components);
            for &tuple_id in tuple_ids {
                values.extend_from_slice($array.tuple(tuple_id));
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                $array.name(),
                values,
                number_of_components,
            )))
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.001, 0.001, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = weld_vertices(&m, 0.01);
        assert!(r.points.len() <= 3);
    }
}
