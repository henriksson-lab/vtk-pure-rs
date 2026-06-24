use crate::data::DataSet;
use crate::data::{CellArray, ImageData, Points, PolyData, UnstructuredGrid};
use crate::types::CellType;
use std::collections::HashMap;

/// Extract the outer (boundary) surface geometry from an UnstructuredGrid.
///
/// For 3D cells (tetra, hex, wedge, pyramid), extracts faces that are not
/// shared between two cells (i.e. boundary faces). For 2D cells (triangles,
/// quads), includes them directly. Returns a PolyData with the surface mesh.
pub fn geometry_filter_unstructured(input: &UnstructuredGrid) -> PolyData {
    // Count face occurrences. Key = sorted vertex indices.
    let mut face_count: HashMap<Vec<i64>, Vec<i64>> = HashMap::new();

    for ci in 0..input.num_cells() {
        let ct = input.cell_type(ci);
        let pts = input.cell_points(ci);

        let faces = cell_faces(ct, pts);
        for face in faces {
            let mut key = face.clone();
            key.sort();
            face_count.entry(key).or_insert(face);
        }
    }

    // Also track which faces appear more than once
    let mut face_shared: HashMap<Vec<i64>, usize> = HashMap::new();
    for ci in 0..input.num_cells() {
        let ct = input.cell_type(ci);
        let pts = input.cell_points(ci);
        let faces = cell_faces(ct, pts);
        for face in faces {
            let mut key = face.clone();
            key.sort();
            *face_shared.entry(key).or_insert(0) += 1;
        }
    }

    // Collect boundary faces (count == 1)
    let boundary_faces: Vec<Vec<i64>> = face_shared
        .iter()
        .filter(|(_, &count)| count == 1)
        .map(|(key, _)| face_count[key].clone())
        .collect();

    // Build output, remapping point indices
    let mut point_map: HashMap<i64, i64> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for face in &boundary_faces {
        let mapped: Vec<i64> = face
            .iter()
            .map(|&pid| {
                *point_map.entry(pid).or_insert_with(|| {
                    let idx = out_points.len() as i64;
                    out_points.push(input.point(pid as usize));
                    idx
                })
            })
            .collect();

        // Fan-triangulate if more than 3 vertices
        if mapped.len() == 3 {
            out_polys.push_cell(&mapped);
        } else {
            for i in 1..mapped.len() - 1 {
                out_polys.push_cell(&[mapped[0], mapped[i], mapped[i + 1]]);
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

/// Extract surface from ImageData as a PolyData of the 6 boundary faces.
pub fn geometry_filter_image(input: &ImageData) -> PolyData {
    let dims = input.dimensions();
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Generate all grid points
    let origin = input.origin();
    let spacing = input.spacing();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;

    let point_idx = |i: usize, j: usize, k: usize| -> i64 { (k * ny * nx + j * nx + i) as i64 };

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                points.push([
                    origin[0] + i as f64 * spacing[0],
                    origin[1] + j as f64 * spacing[1],
                    origin[2] + k as f64 * spacing[2],
                ]);
            }
        }
    }

    // 6 boundary faces
    // -X face (i=0)
    for k in 0..nz - 1 {
        for j in 0..ny - 1 {
            polys.push_cell(&[
                point_idx(0, j, k),
                point_idx(0, j, k + 1),
                point_idx(0, j + 1, k + 1),
                point_idx(0, j + 1, k),
            ]);
        }
    }
    // +X face (i=nx-1)
    for k in 0..nz - 1 {
        for j in 0..ny - 1 {
            polys.push_cell(&[
                point_idx(nx - 1, j, k),
                point_idx(nx - 1, j + 1, k),
                point_idx(nx - 1, j + 1, k + 1),
                point_idx(nx - 1, j, k + 1),
            ]);
        }
    }
    // -Y face (j=0)
    for k in 0..nz - 1 {
        for i in 0..nx - 1 {
            polys.push_cell(&[
                point_idx(i, 0, k),
                point_idx(i + 1, 0, k),
                point_idx(i + 1, 0, k + 1),
                point_idx(i, 0, k + 1),
            ]);
        }
    }
    // +Y face (j=ny-1)
    for k in 0..nz - 1 {
        for i in 0..nx - 1 {
            polys.push_cell(&[
                point_idx(i, ny - 1, k),
                point_idx(i, ny - 1, k + 1),
                point_idx(i + 1, ny - 1, k + 1),
                point_idx(i + 1, ny - 1, k),
            ]);
        }
    }
    // -Z face (k=0)
    for j in 0..ny - 1 {
        for i in 0..nx - 1 {
            polys.push_cell(&[
                point_idx(i, j, 0),
                point_idx(i, j + 1, 0),
                point_idx(i + 1, j + 1, 0),
                point_idx(i + 1, j, 0),
            ]);
        }
    }
    // +Z face (k=nz-1)
    for j in 0..ny - 1 {
        for i in 0..nx - 1 {
            polys.push_cell(&[
                point_idx(i, j, nz - 1),
                point_idx(i + 1, j, nz - 1),
                point_idx(i + 1, j + 1, nz - 1),
                point_idx(i, j + 1, nz - 1),
            ]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

/// Get the faces of a cell as lists of point indices.
fn cell_faces(ct: CellType, pts: &[i64]) -> Vec<Vec<i64>> {
    match ct {
        CellType::Tetra => {
            if pts.len() < 4 {
                return vec![];
            }
            vec![
                vec![pts[0], pts[1], pts[2]],
                vec![pts[0], pts[3], pts[1]],
                vec![pts[1], pts[3], pts[2]],
                vec![pts[0], pts[2], pts[3]],
            ]
        }
        CellType::Hexahedron => {
            if pts.len() < 8 {
                return vec![];
            }
            vec![
                vec![pts[0], pts[3], pts[2], pts[1]], // bottom
                vec![pts[4], pts[5], pts[6], pts[7]], // top
                vec![pts[0], pts[1], pts[5], pts[4]], // front
                vec![pts[2], pts[3], pts[7], pts[6]], // back
                vec![pts[0], pts[4], pts[7], pts[3]], // left
                vec![pts[1], pts[2], pts[6], pts[5]], // right
            ]
        }
        CellType::Wedge => {
            if pts.len() < 6 {
                return vec![];
            }
            vec![
                vec![pts[0], pts[2], pts[1]],         // bottom tri
                vec![pts[3], pts[4], pts[5]],         // top tri
                vec![pts[0], pts[1], pts[4], pts[3]], // quad
                vec![pts[1], pts[2], pts[5], pts[4]], // quad
                vec![pts[0], pts[3], pts[5], pts[2]], // quad
            ]
        }
        CellType::Pyramid => {
            if pts.len() < 5 {
                return vec![];
            }
            vec![
                vec![pts[0], pts[3], pts[2], pts[1]], // base quad
                vec![pts[0], pts[1], pts[4]],         // tri
                vec![pts[1], pts[2], pts[4]],         // tri
                vec![pts[2], pts[3], pts[4]],         // tri
                vec![pts[3], pts[0], pts[4]],         // tri
            ]
        }
        CellType::Triangle => {
            if pts.len() < 3 {
                return vec![];
            }
            vec![vec![pts[0], pts[1], pts[2]]]
        }
        CellType::Quad => {
            if pts.len() < 4 {
                return vec![];
            }
            vec![vec![pts[0], pts[1], pts[2], pts[3]]]
        }
        CellType::Polygon => {
            vec![pts.to_vec()]
        }
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_surface_single_tet() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.points.push([0.0, 0.0, 1.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        let result = geometry_filter_unstructured(&grid);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.polys.num_cells(), 4); // 4 faces
    }

    #[test]
    fn extract_surface_two_tets_shared_face() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]); // 0
        grid.points.push([1.0, 0.0, 0.0]); // 1
        grid.points.push([0.0, 1.0, 0.0]); // 2
        grid.points.push([0.0, 0.0, 1.0]); // 3
        grid.points.push([1.0, 1.0, 1.0]); // 4

        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);
        grid.push_cell(CellType::Tetra, &[1, 2, 3, 4]);

        let result = geometry_filter_unstructured(&grid);
        // 2 tets share face (1,2,3). Total faces = 8, shared = 2 (one from each tet)
        // Boundary faces = 6
        assert_eq!(result.polys.num_cells(), 6);
    }

    #[test]
    fn image_data_surface() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let result = geometry_filter_image(&img);
        // 3×3×3 grid has 27 points
        assert_eq!(result.points.len(), 27);
        // Each face of the cube: 2×2 = 4 quads per face, 6 faces = 24 quads
        assert_eq!(result.polys.num_cells(), 24);
    }

    #[test]
    fn image_data_surface_2x2x2() {
        let img = ImageData::with_dimensions(2, 2, 2);
        let result = geometry_filter_image(&img);
        assert_eq!(result.points.len(), 8);
        // Each face has 1 quad, 6 faces = 6 quads
        assert_eq!(result.polys.num_cells(), 6);
    }
}
