//! Remove degenerate (zero-area) triangles.
//!
//! Vertex merging and isolated-point removal live in
//! [`crate::filters::mesh::vertex_merge_by_distance`] and
//! [`crate::filters::mesh::mesh_remove_isolated`]; they are re-exported here so
//! the historical paths keep working.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, PolyData};

pub use crate::filters::mesh::mesh_remove_isolated::remove_isolated_vertices;
pub use crate::filters::mesh::vertex_merge_by_distance::merge_close_vertices;

/// Remove degenerate triangles with area below threshold.
///
/// A face is dropped when it has fewer than three corners, references a point
/// that does not exist, repeats a corner (vtkCleanPolyData collapses such
/// repeats; here they are treated as degenerate), or - for triangles - spans an
/// area smaller than `min_area`. Faces with more than three corners are measured
/// by the triangle formed by their first three corners.
pub fn remove_degenerate_triangles(mesh: &PolyData, min_area: f64) -> PolyData {
    let mut new_polys = CellArray::new();
    let mut kept_polys = Vec::new();
    for (poly_id, cell) in mesh.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        if !valid_cell_points(cell, mesh.points.len()) {
            continue;
        }
        if has_repeated_point(cell) {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cx = e1[1] * e2[2] - e1[2] * e2[1];
        let cy = e1[2] * e2[0] - e1[0] * e2[2];
        let cz = e1[0] * e2[1] - e1[1] * e2[0];
        let area = 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
        if area >= min_area {
            new_polys.push_cell(cell);
            kept_polys.push(poly_id);
        }
    }
    let mut result = mesh.clone();
    result.polys = new_polys;
    remap_cell_data_for_kept_polys(mesh, &kept_polys, &mut result);
    result
}

fn valid_cell_points(cell: &[i64], num_points: usize) -> bool {
    cell.iter()
        .all(|&id| usize::try_from(id).is_ok_and(|idx| idx < num_points))
}

fn has_repeated_point(cell: &[i64]) -> bool {
    (0..cell.len()).any(|i| cell[i + 1..].contains(&cell[i]))
}

/// Cell data follows the surviving polygons. Cells are numbered verts, lines,
/// polys, strips, and only the poly block changes here.
fn remap_cell_data_for_kept_polys(input: &PolyData, kept_polys: &[usize], output: &mut PolyData) {
    let total_cells = input.total_cells();
    let poly_offset = input.verts.num_cells() + input.lines.num_cells();
    let strip_offset = poly_offset + input.polys.num_cells();
    let mut kept: Vec<usize> = (0..poly_offset).collect();
    kept.extend(kept_polys.iter().map(|&poly_id| poly_offset + poly_id));
    kept.extend(strip_offset..total_cells);

    let mut arrays = Vec::new();
    for array in input.cell_data().iter() {
        if array.num_tuples() == total_cells {
            arrays.push(select_tuples(array, &kept));
        }
    }
    output.cell_data_mut().clear();
    for array in arrays {
        output.cell_data_mut().add_array(array);
    }
    copy_active_attributes(input.cell_data(), output.cell_data_mut());
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

fn select_tuples(array: &AnyDataArray, kept: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {{
            let num_components = $array.num_components();
            let mut data = Vec::with_capacity(kept.len() * num_components);
            for &tuple_id in kept {
                data.extend_from_slice($array.tuple(tuple_id));
            }
            AnyDataArray::$variant(DataArray::from_vec($array.name(), data, num_components))
        }};
    }

    match array {
        AnyDataArray::F32(a) => select!(a, F32),
        AnyDataArray::F64(a) => select!(a, F64),
        AnyDataArray::I8(a) => select!(a, I8),
        AnyDataArray::I16(a) => select!(a, I16),
        AnyDataArray::I32(a) => select!(a, I32),
        AnyDataArray::I64(a) => select!(a, I64),
        AnyDataArray::U8(a) => select!(a, U8),
        AnyDataArray::U16(a) => select!(a, U16),
        AnyDataArray::U32(a) => select!(a, U32),
        AnyDataArray::U64(a) => select!(a, U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_remove_degenerate() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = remove_degenerate_triangles(&mesh, 1e-10);
        assert_eq!(r.polys.num_cells(), 1); // degenerate removed
    }

    #[test]
    fn cell_data_follows_surviving_polygons() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[3, 4, 5], [0, 1, 2]],
        );
        let mut mesh = mesh;
        mesh.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec("ids", vec![7, 9], 1)));

        let r = remove_degenerate_triangles(&mesh, 1e-10);
        let ids = r.cell_data().get_array("ids").unwrap();
        assert_eq!(ids.num_tuples(), 1);
        let mut value = [0.0f64];
        ids.tuple_as_f64(0, &mut value);
        assert_eq!(value[0], 9.0);
    }

    #[test]
    fn drops_faces_that_repeat_a_corner() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2, 1]);

        let r = remove_degenerate_triangles(&mesh, 1e-10);
        assert_eq!(r.polys.num_cells(), 0);
    }
}
