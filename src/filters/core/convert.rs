use crate::data::{
    CellArray, ImageData, Points, PolyData, RectilinearGrid, StructuredGrid, UnstructuredGrid,
};
use crate::types::CellType;

/// Convert an ImageData surface to PolyData quads (outer surface only).
pub fn image_data_surface_to_poly_data(img: &ImageData) -> PolyData {
    let dims = img.dimensions();
    let mut points = Points::new();
    let mut polys = CellArray::new();

    // Generate points
    for k in 0..dims[2] {
        for j in 0..dims[1] {
            for i in 0..dims[0] {
                points.push(img.point_from_ijk(i, j, k));
            }
        }
    }

    let idx =
        |i: usize, j: usize, k: usize| -> i64 { (k * dims[1] * dims[0] + j * dims[0] + i) as i64 };

    // Z-min face (k=0)
    if dims[2] > 0 {
        for j in 0..dims[1] - 1 {
            for i in 0..dims[0] - 1 {
                polys.push_cell(&[
                    idx(i, j, 0),
                    idx(i + 1, j, 0),
                    idx(i + 1, j + 1, 0),
                    idx(i, j + 1, 0),
                ]);
            }
        }
    }
    // Z-max face
    if dims[2] > 1 {
        let k = dims[2] - 1;
        for j in 0..dims[1] - 1 {
            for i in 0..dims[0] - 1 {
                polys.push_cell(&[
                    idx(i, j, k),
                    idx(i, j + 1, k),
                    idx(i + 1, j + 1, k),
                    idx(i + 1, j, k),
                ]);
            }
        }
    }
    // Y-min face
    if dims[1] > 0 {
        for k in 0..dims[2] - 1 {
            for i in 0..dims[0] - 1 {
                polys.push_cell(&[
                    idx(i, 0, k),
                    idx(i + 1, 0, k),
                    idx(i + 1, 0, k + 1),
                    idx(i, 0, k + 1),
                ]);
            }
        }
    }
    // Y-max face
    if dims[1] > 1 {
        let j = dims[1] - 1;
        for k in 0..dims[2] - 1 {
            for i in 0..dims[0] - 1 {
                polys.push_cell(&[
                    idx(i, j, k),
                    idx(i, j, k + 1),
                    idx(i + 1, j, k + 1),
                    idx(i + 1, j, k),
                ]);
            }
        }
    }
    // X-min face
    if dims[0] > 0 {
        for k in 0..dims[2] - 1 {
            for j in 0..dims[1] - 1 {
                polys.push_cell(&[
                    idx(0, j, k),
                    idx(0, j, k + 1),
                    idx(0, j + 1, k + 1),
                    idx(0, j + 1, k),
                ]);
            }
        }
    }
    // X-max face
    if dims[0] > 1 {
        let i = dims[0] - 1;
        for k in 0..dims[2] - 1 {
            for j in 0..dims[1] - 1 {
                polys.push_cell(&[
                    idx(i, j, k),
                    idx(i, j + 1, k),
                    idx(i, j + 1, k + 1),
                    idx(i, j, k + 1),
                ]);
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

/// Convert a PolyData triangle mesh to an UnstructuredGrid.
pub fn poly_data_to_unstructured_grid(pd: &PolyData) -> UnstructuredGrid {
    let mut ug = UnstructuredGrid::new();
    ug.points = pd.points.clone();

    for cell in pd.polys.iter() {
        let ct = match cell.len() {
            3 => CellType::Triangle,
            4 => CellType::Quad,
            _ => CellType::Polygon,
        };
        ug.push_cell(ct, cell);
    }
    for cell in pd.lines.iter() {
        let ct = if cell.len() == 2 {
            CellType::Line
        } else {
            CellType::PolyLine
        };
        ug.push_cell(ct, cell);
    }
    for cell in pd.verts.iter() {
        let ct = if cell.len() == 1 {
            CellType::Vertex
        } else {
            CellType::PolyVertex
        };
        ug.push_cell(ct, cell);
    }

    // Copy point data
    for i in 0..pd.point_data().num_arrays() {
        if let Some(arr) = pd.point_data().get_array_by_index(i) {
            ug.point_data_mut().add_array(arr.clone());
        }
    }

    ug
}

/// Convert an UnstructuredGrid (triangles/quads only) to PolyData.
pub fn unstructured_grid_to_poly_data(ug: &UnstructuredGrid) -> PolyData {
    let mut pd = PolyData::new();
    pd.points = ug.points.clone();

    for i in 0..ug.cells().num_cells() {
        let ct = ug.cell_type(i);
        let pts = ug.cell_points(i);
        match ct.dimension() {
            0 => pd.verts.push_cell(pts),
            1 => pd.lines.push_cell(pts),
            2 | 3 => pd.polys.push_cell(pts),
            _ => {}
        }
    }

    // Copy point data
    for i in 0..ug.point_data().num_arrays() {
        if let Some(arr) = ug.point_data().get_array_by_index(i) {
            pd.point_data_mut().add_array(arr.clone());
        }
    }

    pd
}

/// Convert a RectilinearGrid surface to PolyData.
pub fn rectilinear_grid_to_poly_data(rg: &RectilinearGrid) -> PolyData {
    let dims = rg.dimensions();
    let mut points = Points::new();
    let x = rg.x_coords();
    let y = rg.y_coords();
    let z = rg.z_coords();

    for k in 0..dims[2] {
        for j in 0..dims[1] {
            for i in 0..dims[0] {
                points.push([x[i], y[j], z[k]]);
            }
        }
    }

    let mut polys = CellArray::new();
    let idx =
        |i: usize, j: usize, k: usize| -> i64 { (k * dims[1] * dims[0] + j * dims[0] + i) as i64 };

    // Just do all faces of all cells (no shared face elimination)
    for k in 0..dims[2].saturating_sub(1) {
        for j in 0..dims[1].saturating_sub(1) {
            for i in 0..dims[0].saturating_sub(1) {
                // Top and bottom
                if k == 0 {
                    polys.push_cell(&[
                        idx(i, j, 0),
                        idx(i + 1, j, 0),
                        idx(i + 1, j + 1, 0),
                        idx(i, j + 1, 0),
                    ]);
                }
                if k == dims[2] - 2 {
                    polys.push_cell(&[
                        idx(i, j, k + 1),
                        idx(i, j + 1, k + 1),
                        idx(i + 1, j + 1, k + 1),
                        idx(i + 1, j, k + 1),
                    ]);
                }
                // Front and back
                if j == 0 {
                    polys.push_cell(&[
                        idx(i, 0, k),
                        idx(i + 1, 0, k),
                        idx(i + 1, 0, k + 1),
                        idx(i, 0, k + 1),
                    ]);
                }
                if j == dims[1] - 2 {
                    polys.push_cell(&[
                        idx(i, j + 1, k),
                        idx(i, j + 1, k + 1),
                        idx(i + 1, j + 1, k + 1),
                        idx(i + 1, j + 1, k),
                    ]);
                }
                // Left and right
                if i == 0 {
                    polys.push_cell(&[
                        idx(0, j, k),
                        idx(0, j, k + 1),
                        idx(0, j + 1, k + 1),
                        idx(0, j + 1, k),
                    ]);
                }
                if i == dims[0] - 2 {
                    polys.push_cell(&[
                        idx(i + 1, j, k),
                        idx(i + 1, j + 1, k),
                        idx(i + 1, j + 1, k + 1),
                        idx(i + 1, j, k + 1),
                    ]);
                }
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

/// Convert a StructuredGrid surface to PolyData quads (outer faces).
pub fn structured_grid_to_poly_data(sg: &StructuredGrid) -> PolyData {
    let dims = sg.dimensions();
    let mut pd = PolyData::new();

    // Copy all points
    for i in 0..sg.points.len() {
        pd.points.push(sg.points.get(i));
    }

    let idx =
        |i: usize, j: usize, k: usize| -> i64 { (k * dims[1] * dims[0] + j * dims[0] + i) as i64 };

    // Outer faces only
    for k in 0..dims[2].saturating_sub(1) {
        for j in 0..dims[1].saturating_sub(1) {
            for i in 0..dims[0].saturating_sub(1) {
                if k == 0 {
                    pd.polys.push_cell(&[
                        idx(i, j, 0),
                        idx(i + 1, j, 0),
                        idx(i + 1, j + 1, 0),
                        idx(i, j + 1, 0),
                    ]);
                }
                if k == dims[2].saturating_sub(2) {
                    pd.polys.push_cell(&[
                        idx(i, j, k + 1),
                        idx(i, j + 1, k + 1),
                        idx(i + 1, j + 1, k + 1),
                        idx(i + 1, j, k + 1),
                    ]);
                }
                if j == 0 {
                    pd.polys.push_cell(&[
                        idx(i, 0, k),
                        idx(i + 1, 0, k),
                        idx(i + 1, 0, k + 1),
                        idx(i, 0, k + 1),
                    ]);
                }
                if j == dims[1].saturating_sub(2) {
                    pd.polys.push_cell(&[
                        idx(i, j + 1, k),
                        idx(i, j + 1, k + 1),
                        idx(i + 1, j + 1, k + 1),
                        idx(i + 1, j + 1, k),
                    ]);
                }
                if i == 0 {
                    pd.polys.push_cell(&[
                        idx(0, j, k),
                        idx(0, j, k + 1),
                        idx(0, j + 1, k + 1),
                        idx(0, j + 1, k),
                    ]);
                }
                if i == dims[0].saturating_sub(2) {
                    pd.polys.push_cell(&[
                        idx(i + 1, j, k),
                        idx(i + 1, j + 1, k),
                        idx(i + 1, j + 1, k + 1),
                        idx(i + 1, j, k + 1),
                    ]);
                }
            }
        }
    }

    pd
}

/// Convert an ImageData to a StructuredGrid (explicit point coordinates).
pub fn image_data_to_structured_grid(img: &ImageData) -> StructuredGrid {
    let dims = img.dimensions();
    let mut pts = Points::new();
    for k in 0..dims[2] {
        for j in 0..dims[1] {
            for i in 0..dims[0] {
                pts.push(img.point_from_ijk(i, j, k));
            }
        }
    }
    StructuredGrid::from_dimensions_and_points(dims, pts)
}

/// Convert a StructuredGrid to an ImageData (only if the grid is regular).
///
/// Returns None if the grid is not axis-aligned with uniform spacing.
pub fn structured_grid_to_image_data(sg: &StructuredGrid) -> Option<ImageData> {
    let dims = sg.dimensions();
    if dims[0] < 2 || dims[1] < 2 || dims[2] < 2 {
        return None;
    }

    let p000 = sg.points.get(sg.index_from_ijk(0, 0, 0));
    let p100 = sg.points.get(sg.index_from_ijk(1, 0, 0));
    let p010 = sg.points.get(sg.index_from_ijk(0, 1, 0));
    let p001 = sg.points.get(sg.index_from_ijk(0, 0, 1));

    let spacing = [p100[0] - p000[0], p010[1] - p000[1], p001[2] - p000[2]];

    // Verify it's actually regular
    if spacing[0].abs() < 1e-15 || spacing[1].abs() < 1e-15 || spacing[2].abs() < 1e-15 {
        return None;
    }

    let img = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(spacing)
        .with_origin(p000);
    Some(img)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_to_poly_surface() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let pd = image_data_surface_to_poly_data(&img);
        assert_eq!(pd.points.len(), 27);
        assert!(pd.polys.num_cells() > 0); // should have surface quads
    }

    #[test]
    fn poly_to_unstructured() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let ug = poly_data_to_unstructured_grid(&pd);
        assert_eq!(ug.points.len(), 3);
        assert_eq!(ug.cells().num_cells(), 1);
        assert_eq!(ug.cell_type(0), CellType::Triangle);
    }

    #[test]
    fn unstructured_to_poly() {
        let mut ug = UnstructuredGrid::new();
        ug.points.push([0.0, 0.0, 0.0]);
        ug.points.push([1.0, 0.0, 0.0]);
        ug.points.push([0.0, 1.0, 0.0]);
        ug.push_cell(CellType::Triangle, &[0, 1, 2]);
        let pd = unstructured_grid_to_poly_data(&ug);
        assert_eq!(pd.polys.num_cells(), 1);
    }

    #[test]
    fn rectilinear_to_poly() {
        let rg = RectilinearGrid::from_coords(vec![0.0, 1.0, 2.0], vec![0.0, 1.0], vec![0.0, 1.0]);
        let pd = rectilinear_grid_to_poly_data(&rg);
        assert_eq!(pd.points.len(), 12); // 3*2*2
        assert!(pd.polys.num_cells() > 0);
    }

    #[test]
    fn structured_to_poly() {
        let sg = StructuredGrid::uniform([3, 2, 2], [1.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
        let pd = structured_grid_to_poly_data(&sg);
        assert_eq!(pd.points.len(), 12);
        assert!(pd.polys.num_cells() > 0);
    }

    #[test]
    fn image_to_structured() {
        let img = ImageData::with_dimensions(3, 3, 3)
            .with_spacing([0.5, 0.5, 0.5])
            .with_origin([1.0, 2.0, 3.0]);
        let sg = image_data_to_structured_grid(&img);
        assert_eq!(sg.dimensions(), [3, 3, 3]);
        assert_eq!(sg.points.len(), 27);
        let p = sg.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn roundtrip_image_structured() {
        let img = ImageData::with_dimensions(4, 3, 2)
            .with_spacing([0.5, 1.0, 2.0])
            .with_origin([1.0, 2.0, 3.0]);
        let sg = image_data_to_structured_grid(&img);
        let img2 = structured_grid_to_image_data(&sg).unwrap();
        assert_eq!(img2.dimensions(), img.dimensions());
        assert!((img2.spacing()[0] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn roundtrip_poly_unstructured() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let ug = poly_data_to_unstructured_grid(&pd);
        let pd2 = unstructured_grid_to_poly_data(&ug);
        assert_eq!(pd2.polys.num_cells(), 2);
    }
}
