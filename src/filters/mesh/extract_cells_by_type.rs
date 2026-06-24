//! Extract cells by geometric type (triangles, quads, etc.).

use crate::data::{CellArray, Points, PolyData};

/// Extract only triangles from a mesh.
pub fn extract_triangles(mesh: &PolyData) -> PolyData {
    extract_by_vertex_count(mesh, 3)
}

/// Extract only quads from a mesh.
pub fn extract_quads(mesh: &PolyData) -> PolyData {
    extract_by_vertex_count(mesh, 4)
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
}
