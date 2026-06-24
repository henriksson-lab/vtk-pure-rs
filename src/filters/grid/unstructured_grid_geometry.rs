//! Extract specific faces from an UnstructuredGrid.

use crate::data::{CellArray, Points, PolyData, UnstructuredGrid};
use crate::types::CellType;

/// Extract boundary faces from an UnstructuredGrid as PolyData.
///
/// A boundary face is a face that is not shared by two cells.
pub fn extract_boundary_faces(grid: &UnstructuredGrid) -> PolyData {
    // Build face → cell count map
    let mut face_count: std::collections::HashMap<Vec<usize>, usize> =
        std::collections::HashMap::new();
    let mut all_faces: Vec<Vec<usize>> = Vec::new();

    for cell in grid.cells().iter() {
        let faces = cell_faces(cell);
        for face in faces {
            let mut sorted = face.clone();
            sorted.sort();
            *face_count.entry(sorted).or_insert(0) += 1;
            all_faces.push(face);
        }
    }

    // Collect boundary faces (count == 1)
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut point_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for face in &all_faces {
        let mut sorted = face.clone();
        sorted.sort();
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
                idx
            });
            ids.push(new_idx as i64);
        }
        if ids.len() >= 3 {
            polys.push_cell(&ids);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

/// Extract all faces (including internal) from an UnstructuredGrid.
pub fn extract_all_faces(grid: &UnstructuredGrid) -> PolyData {
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut point_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut seen_faces: std::collections::HashSet<Vec<usize>> = std::collections::HashSet::new();

    for cell in grid.cells().iter() {
        let faces = cell_faces(cell);
        for face in faces {
            let mut sorted = face.clone();
            sorted.sort();
            if !seen_faces.insert(sorted) {
                continue;
            }

            let mut ids = Vec::new();
            for &pid in &face {
                let new_idx = *point_map.entry(pid).or_insert_with(|| {
                    let idx = points.len();
                    points.push(grid.points.get(pid));
                    idx
                });
                ids.push(new_idx as i64);
            }
            if ids.len() >= 3 {
                polys.push_cell(&ids);
            }
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

/// Extract faces of cells with a specific type.
pub fn extract_faces_by_cell_type(grid: &UnstructuredGrid, cell_type: CellType) -> PolyData {
    let types = grid.cell_types();
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut point_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for (ci, cell) in grid.cells().iter().enumerate() {
        if ci >= types.len() || types[ci] != cell_type {
            continue;
        }
        let faces = cell_faces(cell);
        for face in faces {
            let mut ids = Vec::new();
            for &pid in &face {
                let new_idx = *point_map.entry(pid).or_insert_with(|| {
                    let idx = points.len();
                    points.push(grid.points.get(pid));
                    idx
                });
                ids.push(new_idx as i64);
            }
            if ids.len() >= 3 {
                polys.push_cell(&ids);
            }
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

fn cell_faces(cell: &[i64]) -> Vec<Vec<usize>> {
    let n = cell.len();
    match n {
        4 => {
            // Tetrahedron: 4 triangular faces
            let c: Vec<usize> = cell.iter().map(|&i| i as usize).collect();
            vec![
                vec![c[0], c[1], c[2]],
                vec![c[0], c[1], c[3]],
                vec![c[0], c[2], c[3]],
                vec![c[1], c[2], c[3]],
            ]
        }
        8 => {
            // Hexahedron: 6 quad faces
            let c: Vec<usize> = cell.iter().map(|&i| i as usize).collect();
            vec![
                vec![c[0], c[1], c[2], c[3]], // bottom
                vec![c[4], c[5], c[6], c[7]], // top
                vec![c[0], c[1], c[5], c[4]], // front
                vec![c[2], c[3], c[7], c[6]], // back
                vec![c[0], c[3], c[7], c[4]], // left
                vec![c[1], c[2], c[6], c[5]], // right
            ]
        }
        3 => {
            // Triangle: 1 face (itself)
            vec![cell.iter().map(|&i| i as usize).collect()]
        }
        _ => {
            // General polygon: 1 face
            if n >= 3 {
                vec![cell.iter().map(|&i| i as usize).collect()]
            } else {
                Vec::new()
            }
        }
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
