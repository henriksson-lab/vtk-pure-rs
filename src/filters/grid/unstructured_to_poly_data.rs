use crate::data::{CellArray, DataSet, Points, PolyData, UnstructuredGrid};
use crate::types::CellType;

/// Convert an UnstructuredGrid to PolyData by extracting surface cells.
///
/// Keeps only 2D cells (triangles, quads, polygons) and extracts
/// boundary faces from 3D cells using the same logic as `geometry_filter`.
pub fn unstructured_to_poly_data(input: &UnstructuredGrid) -> PolyData {
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();
    let mut pt_map = std::collections::HashMap::new();

    let mut map_point = |id: i64, input: &UnstructuredGrid, out: &mut Points<f64>| -> i64 {
        *pt_map.entry(id).or_insert_with(|| {
            let idx = out.len() as i64;
            out.push(input.point(id as usize));
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
                }
            }
            CellType::Quad | CellType::Polygon => {
                if pts.len() >= 3 {
                    let mapped: Vec<i64> = pts
                        .iter()
                        .map(|&id| map_point(id, input, &mut out_points))
                        .collect();
                    out_polys.push_cell(&mapped);
                }
            }
            CellType::Tetra => {
                // Extract 4 faces
                if pts.len() >= 4 {
                    let faces = [[0, 1, 2], [0, 3, 1], [1, 3, 2], [0, 2, 3]];
                    for f in &faces {
                        let mapped: Vec<i64> = f
                            .iter()
                            .map(|&fi| map_point(pts[fi], input, &mut out_points))
                            .collect();
                        out_polys.push_cell(&mapped);
                    }
                }
            }
            CellType::Hexahedron => {
                if pts.len() >= 8 {
                    let faces = [
                        [0, 3, 2, 1],
                        [4, 5, 6, 7],
                        [0, 1, 5, 4],
                        [2, 3, 7, 6],
                        [0, 4, 7, 3],
                        [1, 2, 6, 5],
                    ];
                    for f in &faces {
                        let mapped: Vec<i64> = f
                            .iter()
                            .map(|&fi| map_point(pts[fi], input, &mut out_points))
                            .collect();
                        out_polys.push_cell(&mapped);
                    }
                }
            }
            _ => {} // skip other cell types
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
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
}
