//! Extract specific faces from an UnstructuredGrid.

use std::collections::{HashMap, HashSet};

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData, UnstructuredGrid};
use crate::types::CellType;

/// Extract boundary faces from an UnstructuredGrid as PolyData.
///
/// A boundary face is a face that is not shared by two cells.
pub fn extract_boundary_faces(grid: &UnstructuredGrid) -> PolyData {
    let mut face_count: HashMap<Vec<usize>, usize> = HashMap::new();
    let mut all_faces: Vec<(Vec<usize>, usize)> = Vec::new();

    for (cell_id, cell) in grid.cells().iter().enumerate() {
        let faces = cell_faces(grid.cell_type(cell_id), cell);
        for face in faces {
            let mut sorted = face.clone();
            sorted.sort_unstable();
            *face_count.entry(sorted).or_insert(0) += 1;
            all_faces.push((face, cell_id));
        }
    }

    // Collect boundary faces (count == 1)
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut point_map: HashMap<usize, usize> = HashMap::new();
    let mut old_point_ids = Vec::new();
    let mut source_cell_ids = Vec::new();

    for (face, source_cell_id) in &all_faces {
        let mut sorted = face.clone();
        sorted.sort_unstable();
        if face_count.get(&sorted) != Some(&1) {
            continue;
        }
        // Mark as processed to avoid duplicates
        face_count.insert(sorted, 0);

        let mut ids = Vec::new();
        for &pid in face {
            let new_idx = *point_map.entry(pid).or_insert_with(|| {
                let idx = points.len();
                points.push(grid.points.get(pid));
                old_point_ids.push(pid);
                idx
            });
            ids.push(new_idx as i64);
        }
        if ids.len() >= 3 {
            polys.push_cell(&ids);
            source_cell_ids.push(*source_cell_id);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    copy_point_data(grid, &old_point_ids, &mut mesh);
    copy_cell_data(grid, &source_cell_ids, &mut mesh);
    mesh
}

/// Extract all faces (including internal) from an UnstructuredGrid.
pub fn extract_all_faces(grid: &UnstructuredGrid) -> PolyData {
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut point_map: HashMap<usize, usize> = HashMap::new();
    let mut old_point_ids = Vec::new();
    let mut source_cell_ids = Vec::new();
    let mut seen_faces: HashSet<Vec<usize>> = HashSet::new();

    for (cell_id, cell) in grid.cells().iter().enumerate() {
        let faces = cell_faces(grid.cell_type(cell_id), cell);
        for face in faces {
            let mut sorted = face.clone();
            sorted.sort_unstable();
            if !seen_faces.insert(sorted) {
                continue;
            }

            let mut ids = Vec::new();
            for &pid in &face {
                let new_idx = *point_map.entry(pid).or_insert_with(|| {
                    let idx = points.len();
                    points.push(grid.points.get(pid));
                    old_point_ids.push(pid);
                    idx
                });
                ids.push(new_idx as i64);
            }
            if ids.len() >= 3 {
                polys.push_cell(&ids);
                source_cell_ids.push(cell_id);
            }
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    copy_point_data(grid, &old_point_ids, &mut mesh);
    copy_cell_data(grid, &source_cell_ids, &mut mesh);
    mesh
}

/// Extract faces of cells with a specific type.
pub fn extract_faces_by_cell_type(grid: &UnstructuredGrid, cell_type: CellType) -> PolyData {
    let types = grid.cell_types();
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut point_map: HashMap<usize, usize> = HashMap::new();
    let mut old_point_ids = Vec::new();
    let mut source_cell_ids = Vec::new();

    for (ci, cell) in grid.cells().iter().enumerate() {
        if ci >= types.len() || types[ci] != cell_type {
            continue;
        }
        let faces = cell_faces(cell_type, cell);
        for face in faces {
            let mut ids = Vec::new();
            for &pid in &face {
                let new_idx = *point_map.entry(pid).or_insert_with(|| {
                    let idx = points.len();
                    points.push(grid.points.get(pid));
                    old_point_ids.push(pid);
                    idx
                });
                ids.push(new_idx as i64);
            }
            if ids.len() >= 3 {
                polys.push_cell(&ids);
                source_cell_ids.push(ci);
            }
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    copy_point_data(grid, &old_point_ids, &mut mesh);
    copy_cell_data(grid, &source_cell_ids, &mut mesh);
    mesh
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

fn cell_faces(cell_type: CellType, cell: &[i64]) -> Vec<Vec<usize>> {
    let c: Vec<usize> = cell.iter().map(|&i| i as usize).collect();
    match cell_type {
        CellType::Tetra if c.len() >= 4 => {
            vec![
                vec![c[0], c[1], c[3]],
                vec![c[1], c[2], c[3]],
                vec![c[2], c[0], c[3]],
                vec![c[0], c[2], c[1]],
            ]
        }
        CellType::Hexahedron | CellType::Voxel if c.len() >= 8 => {
            vec![
                vec![c[0], c[4], c[7], c[3]],
                vec![c[1], c[2], c[6], c[5]],
                vec![c[0], c[1], c[5], c[4]],
                vec![c[3], c[7], c[6], c[2]],
                vec![c[0], c[3], c[2], c[1]],
                vec![c[4], c[5], c[6], c[7]],
            ]
        }
        CellType::Wedge if c.len() >= 6 => {
            vec![
                vec![c[0], c[2], c[1]],
                vec![c[3], c[4], c[5]],
                vec![c[0], c[1], c[4], c[3]],
                vec![c[1], c[2], c[5], c[4]],
                vec![c[2], c[0], c[3], c[5]],
            ]
        }
        CellType::Pyramid if c.len() >= 5 => {
            vec![
                vec![c[0], c[3], c[2], c[1]],
                vec![c[0], c[1], c[4]],
                vec![c[1], c[2], c[4]],
                vec![c[2], c[3], c[4]],
                vec![c[3], c[0], c[4]],
            ]
        }
        CellType::Triangle if c.len() >= 3 => vec![c],
        CellType::Quad | CellType::Pixel | CellType::Polygon if c.len() >= 3 => vec![c],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_tet_boundary() {
        let grid = UnstructuredGrid::from_tetrahedra(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 1, 2, 3]],
        );
        let boundary = extract_boundary_faces(&grid);
        assert_eq!(boundary.polys.num_cells(), 4); // 4 triangular faces
    }

    #[test]
    fn all_faces() {
        let grid = UnstructuredGrid::from_tetrahedra(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 1, 2, 3]],
        );
        let faces = extract_all_faces(&grid);
        assert_eq!(faces.polys.num_cells(), 4);
    }

    #[test]
    fn two_tets_shared_face() {
        let grid = UnstructuredGrid::from_tetrahedra(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
            ],
            vec![[0, 1, 2, 3], [1, 2, 3, 4]],
        );
        let boundary = extract_boundary_faces(&grid);
        // Shared face (1,2,3) should not appear in boundary
        assert!(boundary.polys.num_cells() < 8);
    }

    #[test]
    fn hex_boundary() {
        let grid = UnstructuredGrid::from_hexahedra(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            vec![[0, 1, 2, 3, 4, 5, 6, 7]],
        );
        let boundary = extract_boundary_faces(&grid);
        assert_eq!(boundary.polys.num_cells(), 6); // 6 quad faces
    }
}
