use crate::data::{AnyDataArray, DataArray, DataSet, UnstructuredGrid};
use crate::types::CellType;

/// Triangulate all cells in an UnstructuredGrid to tetrahedra and triangles.
///
/// Decomposes higher-order 3D cells (hexahedra, wedges, pyramids) into
/// tetrahedra, and 2D cells (quads) into triangles. Already-simplex cells
/// (tetrahedra, triangles) are passed through unchanged.
pub fn data_set_triangulate(input: &UnstructuredGrid) -> UnstructuredGrid {
    let mut output = UnstructuredGrid::new();
    let mut output_cell_sources = Vec::new();

    // Copy all points
    for i in 0..input.num_points() {
        output.points.push(input.point(i));
    }
    *output.point_data_mut() = input.point_data().clone();

    for ci in 0..input.num_cells() {
        let ct = input.cell_type(ci);
        let pts = input.cell_points(ci);

        match ct {
            CellType::Triangle => {
                push_cell(
                    &mut output,
                    &mut output_cell_sources,
                    ci,
                    CellType::Triangle,
                    pts,
                );
            }
            CellType::Tetra => {
                push_cell(
                    &mut output,
                    &mut output_cell_sources,
                    ci,
                    CellType::Tetra,
                    pts,
                );
            }
            CellType::Quad => {
                // Split quad into 2 triangles
                if pts.len() >= 4 {
                    push_cell(
                        &mut output,
                        &mut output_cell_sources,
                        ci,
                        CellType::Triangle,
                        &[pts[0], pts[1], pts[2]],
                    );
                    push_cell(
                        &mut output,
                        &mut output_cell_sources,
                        ci,
                        CellType::Triangle,
                        &[pts[0], pts[2], pts[3]],
                    );
                }
            }
            CellType::Hexahedron => {
                // vtkHexahedron::TriangulateLocalIds(index = 0)
                if pts.len() >= 8 {
                    const TETS: [[usize; 4]; 5] = [
                        [2, 1, 5, 0],
                        [0, 2, 3, 7],
                        [2, 5, 6, 7],
                        [0, 7, 4, 5],
                        [0, 2, 7, 5],
                    ];
                    push_tets(&mut output, &mut output_cell_sources, ci, pts, &TETS);
                }
            }
            CellType::Wedge => {
                // vtkWedge::TriangulateLocalIds
                if pts.len() >= 6 {
                    const TETS: [[usize; 4]; 3] = [[0, 1, 2, 3], [1, 4, 5, 3], [1, 3, 5, 2]];
                    push_tets(&mut output, &mut output_cell_sources, ci, pts, &TETS);
                }
            }
            CellType::Pyramid => {
                // vtkPyramid::TriangulateLocalIds chooses the shorter base diagonal.
                if pts.len() >= 5 {
                    let p0 = input.point(pts[0] as usize);
                    let p1 = input.point(pts[1] as usize);
                    let p2 = input.point(pts[2] as usize);
                    let p3 = input.point(pts[3] as usize);
                    let d1 = distance2(p0, p2);
                    let d2 = distance2(p1, p3);
                    if d1 < d2 {
                        const TETS: [[usize; 4]; 2] = [[0, 1, 2, 4], [0, 2, 3, 4]];
                        push_tets(&mut output, &mut output_cell_sources, ci, pts, &TETS);
                    } else {
                        const TETS: [[usize; 4]; 2] = [[0, 1, 3, 4], [1, 2, 3, 4]];
                        push_tets(&mut output, &mut output_cell_sources, ci, pts, &TETS);
                    }
                }
            }
            CellType::Polygon => {
                // Fan-triangulate
                if pts.len() >= 3 {
                    for i in 1..pts.len() - 1 {
                        push_cell(
                            &mut output,
                            &mut output_cell_sources,
                            ci,
                            CellType::Triangle,
                            &[pts[0], pts[i], pts[i + 1]],
                        );
                    }
                }
            }
            _ => {
                // Pass through unchanged
                push_cell(&mut output, &mut output_cell_sources, ci, ct, pts);
            }
        }
    }

    copy_cell_data(input, &mut output, &output_cell_sources);
    output
}

fn push_cell(
    output: &mut UnstructuredGrid,
    output_cell_sources: &mut Vec<usize>,
    source_cell_id: usize,
    cell_type: CellType,
    point_ids: &[i64],
) {
    output.push_cell(cell_type, point_ids);
    output_cell_sources.push(source_cell_id);
}

fn distance2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

fn push_tets(
    output: &mut UnstructuredGrid,
    output_cell_sources: &mut Vec<usize>,
    source_cell_id: usize,
    pts: &[i64],
    tets: &[[usize; 4]],
) {
    for tet in tets {
        push_cell(
            output,
            output_cell_sources,
            source_cell_id,
            CellType::Tetra,
            &[pts[tet[0]], pts[tet[1]], pts[tet[2]], pts[tet[3]]],
        );
    }
}

fn copy_cell_data(input: &UnstructuredGrid, output: &mut UnstructuredGrid, source_ids: &[usize]) {
    for array_index in 0..input.cell_data().num_arrays() {
        if let Some(array) = input.cell_data().get_array_by_index(array_index) {
            output
                .cell_data_mut()
                .add_array(copy_tuples(array, source_ids));
        }
    }
}

fn copy_tuples(array: &AnyDataArray, source_ids: &[usize]) -> AnyDataArray {
    macro_rules! copy_variant {
        ($array:expr, $variant:ident) => {{
            let mut copied = DataArray::new($array.name(), $array.num_components());
            for &source_id in source_ids {
                copied.push_tuple($array.tuple(source_id));
            }
            AnyDataArray::$variant(copied)
        }};
    }

    match array {
        AnyDataArray::F32(array) => copy_variant!(array, F32),
        AnyDataArray::F64(array) => copy_variant!(array, F64),
        AnyDataArray::I8(array) => copy_variant!(array, I8),
        AnyDataArray::I16(array) => copy_variant!(array, I16),
        AnyDataArray::I32(array) => copy_variant!(array, I32),
        AnyDataArray::I64(array) => copy_variant!(array, I64),
        AnyDataArray::U8(array) => copy_variant!(array, U8),
        AnyDataArray::U16(array) => copy_variant!(array, U16),
        AnyDataArray::U32(array) => copy_variant!(array, U32),
        AnyDataArray::U64(array) => copy_variant!(array, U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangulate_hex() {
        let mut grid = UnstructuredGrid::new();
        // Unit cube hex
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([1.0, 1.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.points.push([0.0, 0.0, 1.0]);
        grid.points.push([1.0, 0.0, 1.0]);
        grid.points.push([1.0, 1.0, 1.0]);
        grid.points.push([0.0, 1.0, 1.0]);
        grid.push_cell(CellType::Hexahedron, &[0, 1, 2, 3, 4, 5, 6, 7]);

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 5); // 1 hex -> 5 tets
        for i in 0..result.num_cells() {
            assert_eq!(result.cell_type(i), CellType::Tetra);
        }
        assert_eq!(result.cell_points(0), &[2, 1, 5, 0]);
        assert_eq!(result.cell_points(4), &[0, 2, 7, 5]);
    }

    #[test]
    fn triangulate_wedge() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.5, 1.0, 0.0]);
        grid.points.push([0.0, 0.0, 1.0]);
        grid.points.push([1.0, 0.0, 1.0]);
        grid.points.push([0.5, 1.0, 1.0]);
        grid.push_cell(CellType::Wedge, &[0, 1, 2, 3, 4, 5]);

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 3); // 1 wedge -> 3 tets
        assert_eq!(result.cell_points(1), &[1, 4, 5, 3]);
        assert_eq!(result.cell_points(2), &[1, 3, 5, 2]);
    }

    #[test]
    fn triangulate_pyramid() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([1.0, 1.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.points.push([0.5, 0.5, 1.0]);
        grid.push_cell(CellType::Pyramid, &[0, 1, 2, 3, 4]);

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 2); // 1 pyramid -> 2 tets
    }

    #[test]
    fn passthrough_tet() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.points.push([0.0, 0.0, 1.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 1);
        assert_eq!(result.cell_type(0), CellType::Tetra);
    }

    #[test]
    fn mixed_cells() {
        let mut grid = UnstructuredGrid::new();
        for i in 0..9 {
            grid.points.push([i as f64, 0.0, 0.0]);
        }
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);
        grid.push_cell(CellType::Quad, &[4, 5, 6, 7]);
        grid.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "point_id",
                (0..9).map(|i| i as f64).collect(),
                1,
            )));
        grid.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "source",
                vec![10.0, 20.0],
                1,
            )));

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 3); // 1 tet + 2 triangles from quad
        assert!(result.point_data().get_array("point_id").is_some());
        let source = result.cell_data().get_array("source").unwrap();
        assert_eq!(source.to_f64_vec(), vec![10.0, 20.0, 20.0]);
    }
}
