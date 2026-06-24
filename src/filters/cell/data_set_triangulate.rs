use crate::data::DataSet;
use crate::data::UnstructuredGrid;
use crate::types::CellType;

/// Triangulate all cells in an UnstructuredGrid to tetrahedra and triangles.
///
/// Decomposes higher-order 3D cells (hexahedra, wedges, pyramids) into
/// tetrahedra, and 2D cells (quads) into triangles. Already-simplex cells
/// (tetrahedra, triangles) are passed through unchanged.
pub fn data_set_triangulate(input: &UnstructuredGrid) -> UnstructuredGrid {
    let mut output = UnstructuredGrid::new();

    // Copy all points
    for i in 0..input.num_points() {
        output.points.push(input.point(i));
    }

    for ci in 0..input.num_cells() {
        let ct = input.cell_type(ci);
        let pts = input.cell_points(ci);

        match ct {
            CellType::Triangle => {
                output.push_cell(CellType::Triangle, pts);
            }
            CellType::Tetra => {
                output.push_cell(CellType::Tetra, pts);
            }
            CellType::Quad => {
                // Split quad into 2 triangles
                if pts.len() >= 4 {
                    output.push_cell(CellType::Triangle, &[pts[0], pts[1], pts[2]]);
                    output.push_cell(CellType::Triangle, &[pts[0], pts[2], pts[3]]);
                }
            }
            CellType::Hexahedron => {
                // Split hex into 5 tetrahedra (standard decomposition)
                if pts.len() >= 8 {
                    // Using the Julien Dompierre decomposition
                    output.push_cell(CellType::Tetra, &[pts[0], pts[1], pts[3], pts[4]]);
                    output.push_cell(CellType::Tetra, &[pts[1], pts[2], pts[3], pts[6]]);
                    output.push_cell(CellType::Tetra, &[pts[4], pts[5], pts[6], pts[1]]);
                    output.push_cell(CellType::Tetra, &[pts[4], pts[7], pts[6], pts[3]]);
                    output.push_cell(CellType::Tetra, &[pts[1], pts[4], pts[6], pts[3]]);
                }
            }
            CellType::Wedge => {
                // Split wedge into 3 tetrahedra
                if pts.len() >= 6 {
                    output.push_cell(CellType::Tetra, &[pts[0], pts[1], pts[2], pts[3]]);
                    output.push_cell(CellType::Tetra, &[pts[1], pts[2], pts[3], pts[4]]);
                    output.push_cell(CellType::Tetra, &[pts[2], pts[3], pts[4], pts[5]]);
                }
            }
            CellType::Pyramid => {
                // Split pyramid into 2 tetrahedra
                if pts.len() >= 5 {
                    output.push_cell(CellType::Tetra, &[pts[0], pts[1], pts[3], pts[4]]);
                    output.push_cell(CellType::Tetra, &[pts[1], pts[2], pts[3], pts[4]]);
                }
            }
            CellType::Polygon => {
                // Fan-triangulate
                if pts.len() >= 3 {
                    for i in 1..pts.len() - 1 {
                        output.push_cell(CellType::Triangle, &[pts[0], pts[i], pts[i + 1]]);
                    }
                }
            }
            _ => {
                // Pass through unchanged
                output.push_cell(ct, pts);
            }
        }
    }

    output
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

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 3); // 1 tet + 2 triangles from quad
    }
}
