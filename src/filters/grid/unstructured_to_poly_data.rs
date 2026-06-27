use std::collections::HashMap;

use crate::data::{
    AnyDataArray, CellArray, DataArray, DataSet, Points, PolyData, UnstructuredGrid,
};
use crate::types::CellType;

/// Convert an UnstructuredGrid to PolyData by extracting surface cells.
///
/// Keeps only 2D cells (triangles, quads, polygons) and extracts
/// boundary faces from 3D cells using the same logic as `geometry_filter`.
pub fn unstructured_to_poly_data(input: &UnstructuredGrid) -> PolyData {
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();
    let mut pt_map = std::collections::HashMap::new();
    let mut old_point_ids = Vec::new();
    let mut source_cell_ids = Vec::new();
    let mut face_usage: HashMap<Vec<i64>, (Vec<i64>, usize, usize)> = HashMap::new();

    let mut map_point = |id: i64, input: &UnstructuredGrid, out: &mut Points<f64>| -> i64 {
        *pt_map.entry(id).or_insert_with(|| {
            let idx = out.len() as i64;
            out.push(input.point(id as usize));
            old_point_ids.push(id as usize);
            idx
        })
    };

    for ci in 0..input.num_cells() {
        let ct = input.cell_type(ci);
        let pts = input.cell_points(ci);

        match ct {
            CellType::Triangle => {
                if pts.len() >= 3 {
                    let mapped: Vec<i64> = pts
                        .iter()
                        .map(|&id| map_point(id, input, &mut out_points))
                        .collect();
                    out_polys.push_cell(&mapped);
                    source_cell_ids.push(ci);
                }
            }
            CellType::Quad | CellType::Polygon => {
                if pts.len() >= 3 {
                    let mapped: Vec<i64> = pts
                        .iter()
                        .map(|&id| map_point(id, input, &mut out_points))
                        .collect();
                    out_polys.push_cell(&mapped);
                    source_cell_ids.push(ci);
                }
            }
            CellType::Tetra
            | CellType::Hexahedron
            | CellType::Voxel
            | CellType::Wedge
            | CellType::Pyramid => {
                for face in cell_faces(ct, pts) {
                    let mut key = face.clone();
                    key.sort_unstable();
                    face_usage
                        .entry(key)
                        .and_modify(|(_, count, _)| *count += 1)
                        .or_insert((face, 1, ci));
                }
            }
            _ => {} // skip other cell types
        }
    }

    for (face, count, source_cell_id) in face_usage.values() {
        if *count != 1 {
            continue;
        }
        let mapped: Vec<i64> = face
            .iter()
            .map(|&id| map_point(id, input, &mut out_points))
            .collect();
        out_polys.push_cell(&mapped);
        source_cell_ids.push(*source_cell_id);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    copy_point_data(input, &old_point_ids, &mut pd);
    copy_cell_data(input, &source_cell_ids, &mut pd);
    pd
}

fn cell_faces(cell_type: CellType, pts: &[i64]) -> Vec<Vec<i64>> {
    match cell_type {
        CellType::Tetra if pts.len() >= 4 => {
            vec![
                vec![pts[0], pts[1], pts[3]],
                vec![pts[1], pts[2], pts[3]],
                vec![pts[2], pts[0], pts[3]],
                vec![pts[0], pts[2], pts[1]],
            ]
        }
        CellType::Hexahedron | CellType::Voxel if pts.len() >= 8 => {
            vec![
                vec![pts[0], pts[4], pts[7], pts[3]],
                vec![pts[1], pts[2], pts[6], pts[5]],
                vec![pts[0], pts[1], pts[5], pts[4]],
                vec![pts[3], pts[7], pts[6], pts[2]],
                vec![pts[0], pts[3], pts[2], pts[1]],
                vec![pts[4], pts[5], pts[6], pts[7]],
            ]
        }
        CellType::Wedge if pts.len() >= 6 => {
            vec![
                vec![pts[0], pts[2], pts[1]],
                vec![pts[3], pts[4], pts[5]],
                vec![pts[0], pts[1], pts[4], pts[3]],
                vec![pts[1], pts[2], pts[5], pts[4]],
                vec![pts[2], pts[0], pts[3], pts[5]],
            ]
        }
        CellType::Pyramid if pts.len() >= 5 => {
            vec![
                vec![pts[0], pts[3], pts[2], pts[1]],
                vec![pts[0], pts[1], pts[4]],
                vec![pts[1], pts[2], pts[4]],
                vec![pts[2], pts[3], pts[4]],
                vec![pts[3], pts[0], pts[4]],
            ]
        }
        _ => Vec::new(),
    }
}

fn copy_point_data(input: &UnstructuredGrid, old_point_ids: &[usize], output: &mut PolyData) {
    for i in 0..input.point_data().num_arrays() {
        let Some(array) = input.point_data().get_array_by_index(i) else {
            continue;
        };
        if array.num_tuples() == input.num_points() {
            output
                .point_data_mut()
                .add_array(select_tuples(array, old_point_ids));
        }
    }
}

fn copy_cell_data(input: &UnstructuredGrid, source_cell_ids: &[usize], output: &mut PolyData) {
    for i in 0..input.cell_data().num_arrays() {
        let Some(array) = input.cell_data().get_array_by_index(i) else {
            continue;
        };
        if array.num_tuples() == input.num_cells() {
            output
                .cell_data_mut()
                .add_array(select_tuples(array, source_cell_ids));
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_tet() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.points.push([0.0, 0.0, 1.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        let pd = unstructured_to_poly_data(&grid);
        assert_eq!(pd.polys.num_cells(), 4); // 4 faces
        assert_eq!(pd.points.len(), 4);
    }

    #[test]
    fn triangle_passthrough() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.push_cell(CellType::Triangle, &[0, 1, 2]);

        let pd = unstructured_to_poly_data(&grid);
        assert_eq!(pd.polys.num_cells(), 1);
    }

    #[test]
    fn empty_grid() {
        let grid = UnstructuredGrid::new();
        let pd = unstructured_to_poly_data(&grid);
        assert_eq!(pd.polys.num_cells(), 0);
    }

    #[test]
    fn two_tets_drop_shared_face() {
        let mut grid = UnstructuredGrid::new();
        for point in [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
        ] {
            grid.points.push(point);
        }
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);
        grid.push_cell(CellType::Tetra, &[1, 2, 3, 4]);

        let pd = unstructured_to_poly_data(&grid);
        assert_eq!(pd.polys.num_cells(), 6);
    }
}
