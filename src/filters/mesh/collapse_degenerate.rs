//! Remove degenerate (zero-area) triangles and duplicate vertices.

use crate::data::{CellArray, Points, PolyData};

/// Remove degenerate triangles with area below threshold.
pub fn remove_degenerate_triangles(mesh: &PolyData, min_area: f64) -> PolyData {
    let mut new_polys = CellArray::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cx = e1[1] * e2[2] - e1[2] * e2[1];
        let cy = e1[2] * e2[0] - e1[0] * e2[2];
        let cz = e1[0] * e2[1] - e1[1] * e2[0];
        let area = 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
        if area >= min_area {
            new_polys.push_cell(cell);
        }
    }
    let mut result = mesh.clone();
    result.polys = new_polys;
    result
}

/// Merge duplicate vertices within tolerance distance.
pub fn merge_close_vertices(mesh: &PolyData, tolerance: f64) -> PolyData {
    let n = mesh.points.len();
    let tol2 = tolerance * tolerance;
    let mut remap = vec![0usize; n];
    let mut new_pts = Points::<f64>::new();
    let mut new_indices: Vec<usize> = Vec::new();

    for i in 0..n {
        let p = mesh.points.get(i);
        let mut found = false;
        for (j, &ni) in new_indices.iter().enumerate() {
            let q = mesh.points.get(ni);
            let d2 = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
            if d2 < tol2 {
                remap[i] = j;
                found = true;
                break;
            }
        }
        if !found {
            remap[i] = new_pts.len();
            new_pts.push(p);
            new_indices.push(i);
        }
    }

    let mut new_polys = CellArray::new();
    for cell in mesh.polys.iter() {
        let mapped: Vec<i64> = cell.iter().map(|&v| remap[v as usize] as i64).collect();
        // Skip degenerate after merge
        let unique: std::collections::HashSet<i64> = mapped.iter().copied().collect();
        if unique.len() >= 3 {
            new_polys.push_cell(&mapped);
        }
    }

    let mut result = PolyData::new();
    result.points = new_pts;
    result.polys = new_polys;
    result
}

/// Remove isolated vertices (not referenced by any cell).
pub fn remove_isolated_vertices(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut used = vec![false; n];
    for cell in mesh.polys.iter() {
        for &v in cell {
            used[v as usize] = true;
        }
    }
    for cell in mesh.lines.iter() {
        for &v in cell {
            used[v as usize] = true;
        }
    }

    let mut pt_map = vec![0usize; n];
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }

    let remap_cells = |ca: &crate::data::CellArray| -> CellArray {
        let mut new = CellArray::new();
        for cell in ca.iter() {
            let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
            new.push_cell(&mapped);
        }
        new
    };

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = remap_cells(&mesh.polys);
    result.lines = remap_cells(&mesh.lines);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_remove_degenerate() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = remove_degenerate_triangles(&mesh, 1e-10);
        assert_eq!(r.polys.num_cells(), 1); // degenerate removed
    }
    #[test]
    fn test_merge() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.001, 0.0, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = merge_close_vertices(&mesh, 0.01);
        assert!(r.points.len() < 4);
    }
    #[test]
    fn test_isolated() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [99.0, 99.0, 99.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = remove_isolated_vertices(&mesh);
        assert_eq!(r.points.len(), 3);
    }
}
