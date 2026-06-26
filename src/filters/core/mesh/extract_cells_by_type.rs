//! Extract cells by geometric type (triangles, quads, etc.).

use crate::data::{CellArray, Points, PolyData};
use crate::types::CellType;

/// Extract cells of a specific VTK cell type from a PolyData.
pub fn extract_cells_by_type(mesh: &PolyData, cell_type: CellType) -> PolyData {
    extract_cells_by_types(mesh, &[cell_type])
}

/// Extract cells matching any of the requested VTK cell types.
pub fn extract_cells_by_types(mesh: &PolyData, types: &[CellType]) -> PolyData {
    let mut used = vec![false; mesh.points.len()];
    mark_selected_points(&mesh.verts, CellFamily::Verts, types, &mut used);
    mark_selected_points(&mesh.lines, CellFamily::Lines, types, &mut used);
    mark_selected_points(&mesh.polys, CellFamily::Polys, types, &mut used);
    mark_selected_points(&mesh.strips, CellFamily::Strips, types, &mut used);

    let (points, point_map) = compact_points(mesh, &used);

    let mut result = PolyData::new();
    result.points = points;
    result.verts = copy_selected_cells(&mesh.verts, CellFamily::Verts, types, &point_map);
    result.lines = copy_selected_cells(&mesh.lines, CellFamily::Lines, types, &point_map);
    result.polys = copy_selected_cells(&mesh.polys, CellFamily::Polys, types, &point_map);
    result.strips = copy_selected_cells(&mesh.strips, CellFamily::Strips, types, &point_map);
    result
}

#[derive(Clone, Copy)]
enum CellFamily {
    Verts,
    Lines,
    Polys,
    Strips,
}

fn poly_data_cell_type(family: CellFamily, cell: &[i64]) -> CellType {
    match family {
        CellFamily::Verts => {
            if cell.len() == 1 {
                CellType::Vertex
            } else {
                CellType::PolyVertex
            }
        }
        CellFamily::Lines => {
            if cell.len() == 2 {
                CellType::Line
            } else {
                CellType::PolyLine
            }
        }
        CellFamily::Polys => match cell.len() {
            3 => CellType::Triangle,
            4 => CellType::Quad,
            _ => CellType::Polygon,
        },
        CellFamily::Strips => CellType::TriangleStrip,
    }
}

fn mark_selected_points(
    cells: &CellArray,
    family: CellFamily,
    types: &[CellType],
    used: &mut [bool],
) {
    for cell in cells.iter() {
        if types.contains(&poly_data_cell_type(family, cell)) {
            for &id in cell {
                used[id as usize] = true;
            }
        }
    }
}

fn compact_points(mesh: &PolyData, used: &[bool]) -> (Points<f64>, Vec<usize>) {
    let mut point_map = vec![0usize; mesh.points.len()];
    let mut points = Points::<f64>::new();
    for (i, &is_used) in used.iter().enumerate() {
        if is_used {
            point_map[i] = points.len();
            points.push(mesh.points.get(i));
        }
    }
    (points, point_map)
}

fn copy_selected_cells(
    cells: &CellArray,
    family: CellFamily,
    types: &[CellType],
    point_map: &[usize],
) -> CellArray {
    let mut output = CellArray::new();
    for cell in cells.iter() {
        if types.contains(&poly_data_cell_type(family, cell)) {
            let mapped: Vec<i64> = cell.iter().map(|&v| point_map[v as usize] as i64).collect();
            output.push_cell(&mapped);
        }
    }
    output
}

/// Extract only triangles from a mesh.
pub fn extract_triangles(mesh: &PolyData) -> PolyData {
    extract_cells_by_type(mesh, CellType::Triangle)
}

/// Extract only quads from a mesh.
pub fn extract_quads(mesh: &PolyData) -> PolyData {
    extract_cells_by_type(mesh, CellType::Quad)
}

/// Extract cells with exactly `count` vertices.
pub fn extract_by_vertex_count(mesh: &PolyData, count: usize) -> PolyData {
    let mut used = vec![false; mesh.points.len()];
    let mut new_polys = CellArray::new();
    let cells: Vec<Vec<i64>> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == count)
        .map(|c| c.to_vec())
        .collect();
    for cell in &cells {
        for &v in cell {
            used[v as usize] = true;
        }
    }
    let mut pt_map = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    for cell in &cells {
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        new_polys.push_cell(&mapped);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = new_polys;
    result
}

/// Extract cells with vertex count in range [min, max].
pub fn extract_by_vertex_count_range(mesh: &PolyData, min: usize, max: usize) -> PolyData {
    let mut used = vec![false; mesh.points.len()];
    let mut new_polys = CellArray::new();
    let cells: Vec<Vec<i64>> = mesh
        .polys
        .iter()
        .filter(|c| c.len() >= min && c.len() <= max)
        .map(|c| c.to_vec())
        .collect();
    for cell in &cells {
        for &v in cell {
            used[v as usize] = true;
        }
    }
    let mut pt_map = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    for cell in &cells {
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        new_polys.push_cell(&mapped);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = new_polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extract_tris() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        // Add a quad
        mesh.polys.push_cell(&[0, 1, 3, 2]);
        let tris = extract_triangles(&mesh);
        assert_eq!(tris.polys.num_cells(), 1);
        let quads = extract_quads(&mesh);
        assert_eq!(quads.polys.num_cells(), 1);
    }
    #[test]
    fn test_range() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        mesh.polys.push_cell(&[0, 1, 3, 2]);
        let all = extract_by_vertex_count_range(&mesh, 3, 4);
        assert_eq!(all.polys.num_cells(), 2);
    }

    #[test]
    fn test_extract_lines_and_strips_by_vtk_type() {
        let mut mesh = PolyData::new();
        for i in 0..5 {
            mesh.points.push([i as f64, 0.0, 0.0]);
        }
        mesh.lines.push_cell(&[0, 1]);
        mesh.lines.push_cell(&[1, 2, 3]);
        mesh.strips.push_cell(&[0, 1, 2, 3]);

        let lines = extract_cells_by_type(&mesh, CellType::Line);
        assert_eq!(lines.lines.num_cells(), 1);
        assert_eq!(lines.points.len(), 2);

        let polylines = extract_cells_by_type(&mesh, CellType::PolyLine);
        assert_eq!(polylines.lines.num_cells(), 1);
        assert_eq!(polylines.points.len(), 3);

        let strips = extract_cells_by_type(&mesh, CellType::TriangleStrip);
        assert_eq!(strips.strips.num_cells(), 1);
        assert_eq!(strips.points.len(), 4);
    }
}
