use std::collections::HashMap;

use crate::data::{
    AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData, UnstructuredGrid,
};
use crate::types::CellType;

/// Extract the outer surface of an UnstructuredGrid as PolyData.
///
/// Identifies boundary faces (faces used by only one cell) and outputs them
/// as polygons. Works for tetrahedral, hexahedral, wedge, and pyramid cells.
pub fn extract_surface(grid: &UnstructuredGrid) -> PolyData {
    let mut face_usage: HashMap<Vec<i64>, (Vec<i64>, usize, usize)> = HashMap::new();
    for cell_idx in 0..grid.cells().num_cells() {
        let cell_type = grid.cell_type(cell_idx);
        let pts = grid.cell_points(cell_idx);

        let faces = cell_faces(cell_type, pts);
        for face in faces {
            let mut key = face.clone();
            key.sort();
            face_usage
                .entry(key)
                .and_modify(|(_, count, _)| *count += 1)
                .or_insert((face, 1, cell_idx));
        }
    }

    // Boundary faces are used by exactly one cell
    let mut point_map: HashMap<i64, usize> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut old_point_ids = Vec::new();
    let mut old_cell_ids = Vec::new();

    for (face, count, cell_idx) in face_usage.values() {
        if *count != 1 {
            continue;
        }
        let remapped: Vec<i64> = face
            .iter()
            .map(|&id| {
                *point_map.entry(id).or_insert_with(|| {
                    let idx = out_points.len();
                    out_points.push(grid.points.get(id as usize));
                    old_point_ids.push(id as usize);
                    idx
                }) as i64
            })
            .collect();
        polys.push_cell(&remapped);
        old_cell_ids.push(*cell_idx);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = polys;
    copy_point_data(grid, &old_point_ids, &mut pd);
    copy_cell_data(grid, &old_cell_ids, &mut pd);
    pd
}

/// Get the faces of a cell as vectors of point indices.
fn cell_faces(cell_type: CellType, pts: &[i64]) -> Vec<Vec<i64>> {
    match cell_type {
        CellType::Tetra => {
            vec![
                vec![pts[0], pts[1], pts[3]],
                vec![pts[1], pts[2], pts[3]],
                vec![pts[2], pts[0], pts[3]],
                vec![pts[0], pts[2], pts[1]],
            ]
        }
        CellType::Hexahedron => {
            vec![
                vec![pts[0], pts[4], pts[7], pts[3]],
                vec![pts[1], pts[2], pts[6], pts[5]],
                vec![pts[0], pts[1], pts[5], pts[4]],
                vec![pts[3], pts[7], pts[6], pts[2]],
                vec![pts[0], pts[3], pts[2], pts[1]],
                vec![pts[4], pts[5], pts[6], pts[7]],
            ]
        }
        CellType::Wedge => {
            vec![
                vec![pts[0], pts[2], pts[1]],
                vec![pts[3], pts[4], pts[5]],
                vec![pts[0], pts[1], pts[4], pts[3]],
                vec![pts[1], pts[2], pts[5], pts[4]],
                vec![pts[2], pts[0], pts[3], pts[5]],
            ]
        }
        CellType::Pyramid => {
            vec![
                vec![pts[0], pts[3], pts[2], pts[1]],
                vec![pts[0], pts[1], pts[4]],
                vec![pts[1], pts[2], pts[4]],
                vec![pts[2], pts[3], pts[4]],
                vec![pts[3], pts[0], pts[4]],
            ]
        }
        CellType::Triangle => {
            vec![vec![pts[0], pts[1], pts[2]]]
        }
        CellType::Quad => {
            vec![vec![pts[0], pts[1], pts[2], pts[3]]]
        }
        _ => Vec::new(),
    }
}

fn copy_point_data(grid: &UnstructuredGrid, old_point_ids: &[usize], output: &mut PolyData) {
    for array in grid.point_data().iter() {
        if array.num_tuples() == grid.points.len() {
            output
                .point_data_mut()
                .add_array(select_tuples(array, old_point_ids));
        }
    }
    copy_active_attributes(grid.point_data(), output.point_data_mut());
}

fn copy_cell_data(grid: &UnstructuredGrid, old_cell_ids: &[usize], output: &mut PolyData) {
    for array in grid.cell_data().iter() {
        if array.num_tuples() == grid.cells().num_cells() {
            output
                .cell_data_mut()
                .add_array(select_tuples(array, old_cell_ids));
        }
    }
    copy_active_attributes(grid.cell_data(), output.cell_data_mut());
}

fn select_tuples(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {{
            let mut out = DataArray::new($array.name(), $array.num_components());
            for &tuple_id in tuple_ids {
                out.push_tuple($array.tuple(tuple_id));
            }
            AnyDataArray::$variant(out)
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
    fn surface_of_single_tetra() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.5, 1.0, 0.0]);
        grid.points.push([0.5, 0.5, 1.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        let surface = extract_surface(&grid);
        // Single tetra has 4 boundary faces
        assert_eq!(surface.polys.num_cells(), 4);
        assert_eq!(surface.points.len(), 4);
    }

    #[test]
    fn shared_face_removed() {
        let mut grid = UnstructuredGrid::new();
        // Two tetras sharing a face
        grid.points.push([0.0, 0.0, 0.0]); // 0
        grid.points.push([1.0, 0.0, 0.0]); // 1
        grid.points.push([0.5, 1.0, 0.0]); // 2
        grid.points.push([0.5, 0.5, 1.0]); // 3
        grid.points.push([0.5, 0.5, -1.0]); // 4

        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 4]);

        let surface = extract_surface(&grid);
        // 8 total faces - 2 shared (face 0,1,2 from each) = 6 boundary faces
        assert_eq!(surface.polys.num_cells(), 6);
    }
}
