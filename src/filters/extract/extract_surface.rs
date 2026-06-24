use std::collections::HashMap;

use crate::data::{CellArray, Points, PolyData, UnstructuredGrid};
use crate::types::CellType;

/// Extract the outer surface of an UnstructuredGrid as PolyData.
///
/// Identifies boundary faces (faces used by only one cell) and outputs them
/// as polygons. Works for tetrahedral, hexahedral, wedge, and pyramid cells.
pub fn extract_surface(grid: &UnstructuredGrid) -> PolyData {
    // Count how many cells use each face (sorted tuple of point indices)
    let mut face_count: HashMap<Vec<i64>, Vec<i64>> = HashMap::new();

    for cell_idx in 0..grid.cells().num_cells() {
        let cell_type = grid.cell_type(cell_idx);
        let pts = grid.cell_points(cell_idx);

        let faces = cell_faces(cell_type, pts);
        for face in faces {
            let mut key = face.clone();
            key.sort();
            face_count.entry(key).and_modify(|_| {}).or_insert(face);
            // Track count separately
        }
    }

    // Actually need to count occurrences properly
    let mut face_usage: HashMap<Vec<i64>, (Vec<i64>, usize)> = HashMap::new();
    for cell_idx in 0..grid.cells().num_cells() {
        let cell_type = grid.cell_type(cell_idx);
        let pts = grid.cell_points(cell_idx);

        let faces = cell_faces(cell_type, pts);
        for face in faces {
            let mut key = face.clone();
            key.sort();
            face_usage
                .entry(key)
                .and_modify(|(_, count)| *count += 1)
                .or_insert((face, 1));
        }
    }

    // Boundary faces are used by exactly one cell
    let mut point_map: HashMap<i64, usize> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut polys = CellArray::new();

    for (face, count) in face_usage.values() {
        if *count != 1 {
            continue;
        }
        let remapped: Vec<i64> = face
            .iter()
            .map(|&id| {
                *point_map.entry(id).or_insert_with(|| {
                    let idx = out_points.len();
                    out_points.push(grid.points.get(id as usize));
                    idx
                }) as i64
            })
            .collect();
        polys.push_cell(&remapped);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = polys;
    pd
}

/// Get the faces of a cell as vectors of point indices.
fn cell_faces(cell_type: CellType, pts: &[i64]) -> Vec<Vec<i64>> {
    match cell_type {
        CellType::Tetra => {
            vec![
                vec![pts[0], pts[1], pts[2]],
                vec![pts[0], pts[1], pts[3]],
                vec![pts[0], pts[2], pts[3]],
                vec![pts[1], pts[2], pts[3]],
            ]
        }
        CellType::Hexahedron => {
            vec![
                vec![pts[0], pts[1], pts[2], pts[3]], // bottom
                vec![pts[4], pts[5], pts[6], pts[7]], // top
                vec![pts[0], pts[1], pts[5], pts[4]], // front
                vec![pts[2], pts[3], pts[7], pts[6]], // back
                vec![pts[0], pts[3], pts[7], pts[4]], // left
                vec![pts[1], pts[2], pts[6], pts[5]], // right
            ]
        }
        CellType::Wedge => {
            vec![
                vec![pts[0], pts[1], pts[2]],         // bottom tri
                vec![pts[3], pts[4], pts[5]],         // top tri
                vec![pts[0], pts[1], pts[4], pts[3]], // front quad
                vec![pts[1], pts[2], pts[5], pts[4]], // right quad
                vec![pts[0], pts[2], pts[5], pts[3]], // left quad
            ]
        }
        CellType::Pyramid => {
            vec![
                vec![pts[0], pts[1], pts[2], pts[3]], // base quad
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
