use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;
use std::collections::HashMap;

/// Select faces whose normal is within a given angle of a reference direction.
///
/// Returns a new PolyData containing only the faces whose outward normal
/// makes an angle less than or equal to `max_angle_deg` with `direction`.
/// Points are compacted so only referenced vertices are included.
pub fn select_faces_by_normal(
    input: &PolyData,
    direction: [f64; 3],
    max_angle_deg: f64,
) -> PolyData {
    let dir_len: f64 =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    if dir_len < 1e-15 {
        return PolyData::new();
    }
    let dx: f64 = direction[0] / dir_len;
    let dy: f64 = direction[1] / dir_len;
    let dz: f64 = direction[2] / dir_len;
    let cos_threshold: f64 = max_angle_deg.clamp(0.0, 180.0).to_radians().cos();

    let mut new_points = Points::new();
    let mut new_polys = CellArray::new();
    let mut point_map: HashMap<usize, i64> = HashMap::new();
    let mut selected_point_ids = Vec::new();
    let mut selected_cell_ids = Vec::new();
    let poly_cell_offset = input.verts.num_cells() + input.lines.num_cells();
    let use_global_cell_ids = uses_global_cell_ids(input);

    for (cell_id, cell) in input.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        if cell
            .iter()
            .any(|&pid| pid < 0 || (pid as usize) >= input.points.len())
        {
            continue;
        }

        let [nx, ny, nz] = polygon_normal(input, cell);
        let nlen: f64 = (nx * nx + ny * ny + nz * nz).sqrt();

        if nlen < 1e-15 {
            continue;
        }

        let cos_angle: f64 = (nx * dx + ny * dy + nz * dz) / nlen;

        if cos_angle >= cos_threshold {
            // Remap point indices
            let new_cell: Vec<i64> = cell
                .iter()
                .map(|&old_id| {
                    let old: usize = old_id as usize;
                    *point_map.entry(old).or_insert_with(|| {
                        let idx: i64 = new_points.len() as i64;
                        new_points.push(input.points.get(old));
                        selected_point_ids.push(old);
                        idx
                    })
                })
                .collect();
            new_polys.push_cell(&new_cell);
            selected_cell_ids.push(if use_global_cell_ids {
                poly_cell_offset + cell_id
            } else {
                cell_id
            });
        }
    }

    let mut pd = PolyData::new();
    pd.points = new_points;
    pd.polys = new_polys;
    *pd.field_data_mut() = input.field_data().clone();
    copy_attributes_by_indices(input.point_data(), pd.point_data_mut(), &selected_point_ids);
    copy_attributes_by_indices(input.cell_data(), pd.cell_data_mut(), &selected_cell_ids);
    pd
}

fn polygon_normal(input: &PolyData, cell: &[i64]) -> [f64; 3] {
    let mut normal = [0.0, 0.0, 0.0];
    for i in 0..cell.len() {
        let current = input.points.get(cell[i] as usize);
        let next = input.points.get(cell[(i + 1) % cell.len()] as usize);
        normal[0] += (current[1] - next[1]) * (current[2] + next[2]);
        normal[1] += (current[2] - next[2]) * (current[0] + next[0]);
        normal[2] += (current[0] - next[0]) * (current[1] + next[1]);
    }
    normal
}

fn uses_global_cell_ids(mesh: &PolyData) -> bool {
    mesh.cell_data()
        .iter()
        .any(|array| array.num_tuples() >= mesh.total_cells())
}

fn copy_attributes_by_indices(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    indices: &[usize],
) {
    for array in source.iter() {
        if indices.iter().all(|&idx| idx < array.num_tuples()) {
            target.add_array(copy_array_by_indices(array, indices));
        }
    }
    copy_active_attributes(source, target);
}

fn copy_array_by_indices(array: &AnyDataArray, indices: &[usize]) -> AnyDataArray {
    macro_rules! copy {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(copy_typed_array($array, indices))
        };
    }
    match array {
        AnyDataArray::F32(a) => copy!(a, F32),
        AnyDataArray::F64(a) => copy!(a, F64),
        AnyDataArray::I8(a) => copy!(a, I8),
        AnyDataArray::I16(a) => copy!(a, I16),
        AnyDataArray::I32(a) => copy!(a, I32),
        AnyDataArray::I64(a) => copy!(a, I64),
        AnyDataArray::U8(a) => copy!(a, U8),
        AnyDataArray::U16(a) => copy!(a, U16),
        AnyDataArray::U32(a) => copy!(a, U32),
        AnyDataArray::U64(a) => copy!(a, U64),
    }
}

fn copy_typed_array<T: Scalar>(array: &DataArray<T>, indices: &[usize]) -> DataArray<T> {
    let num_components = array.num_components();
    let mut data = Vec::with_capacity(indices.len() * num_components);
    for &idx in indices {
        data.extend_from_slice(array.tuple(idx));
    }
    DataArray::from_vec(array.name(), data, num_components)
}

fn copy_active_attributes(source: &DataSetAttributes, target: &mut DataSetAttributes) {
    if let Some(array) = source.scalars() {
        if target.get_array(array.name()).is_some() {
            target.set_active_scalars(array.name());
        }
    }
    if let Some(array) = source.vectors() {
        if target.get_array(array.name()).is_some() {
            target.set_active_vectors(array.name());
        }
    }
    if let Some(array) = source.normals() {
        if target.get_array(array.name()).is_some() {
            target.set_active_normals(array.name());
        }
    }
    if let Some(array) = source.tcoords() {
        if target.get_array(array.name()).is_some() {
            target.set_active_tcoords(array.name());
        }
    }
    if let Some(array) = source.tensors() {
        if target.get_array(array.name()).is_some() {
            target.set_active_tensors(array.name());
        }
    }
    if let Some(array) = source.global_ids() {
        if target.get_array(array.name()).is_some() {
            target.set_active_global_ids(array.name());
        }
    }
    if let Some(array) = source.pedigree_ids() {
        if target.get_array(array.name()).is_some() {
            target.set_active_pedigree_ids(array.name());
        }
    }
    if let Some(array) = source.edge_flags() {
        if target.get_array(array.name()).is_some() {
            target.set_active_edge_flags(array.name());
        }
    }
    if let Some(array) = source.tangents() {
        if target.get_array(array.name()).is_some() {
            target.set_active_tangents(array.name());
        }
    }
    if let Some(array) = source.rational_weights() {
        if target.get_array(array.name()).is_some() {
            target.set_active_rational_weights(array.name());
        }
    }
    if let Some(array) = source.higher_order_degrees() {
        if target.get_array(array.name()).is_some() {
            target.set_active_higher_order_degrees(array.name());
        }
    }
    if let Some(array) = source.process_ids() {
        if target.get_array(array.name()).is_some() {
            target.set_active_process_ids(array.name());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn select_upward_faces() {
        // Two triangles: one facing up (+z), one facing down (-z)
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0], // +z normal
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 0.0, 1.0], // -z normal (reversed winding)
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = select_faces_by_normal(&pd, [0.0, 0.0, 1.0], 45.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn wide_angle_selects_all() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = select_faces_by_normal(&pd, [0.0, 0.0, 1.0], 180.0);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn zero_angle_exact_match() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        // Normal is exactly +z, selecting with direction +z and angle 0
        let result = select_faces_by_normal(&pd, [0.0, 0.0, 1.0], 0.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn preserves_selected_attributes() {
        let mut pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temperature",
                vec![10.0, 11.0, 12.0, 20.0, 21.0, 22.0],
                1,
            )));
        pd.point_data_mut().set_active_scalars("temperature");
        pd.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cell_id",
                vec![7, 8],
                1,
            )));

        let result = select_faces_by_normal(&pd, [0.0, 0.0, 1.0], 45.0);

        let point_values = result.point_data().get_array("temperature").unwrap();
        let mut buf = [0.0f64];
        point_values.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 10.0);
        assert!(result.point_data().scalars().is_some());

        let cell_values = result.cell_data().get_array("cell_id").unwrap();
        cell_values.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 7.0);
    }

    #[test]
    fn selects_polygon_when_first_three_points_are_collinear() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.5, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3, 4]);

        let result = select_faces_by_normal(&pd, [0.0, 0.0, 1.0], 0.0);

        assert_eq!(result.polys.num_cells(), 1);
    }
}
