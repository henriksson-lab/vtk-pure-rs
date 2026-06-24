//! Convert between cell and vertex representations.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Convert triangles to independent vertices (no sharing).
///
/// Each triangle gets its own 3 vertices. Useful for flat shading.
pub fn cells_to_independent_vertices(mesh: &PolyData) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for cell in mesh.polys.iter() {
        let base = pts.len() as i64;
        let ids: Vec<i64> = cell
            .iter()
            .enumerate()
            .map(|(i, &pid)| {
                pts.push(mesh.points.get(pid as usize));
                base + i as i64
            })
            .collect();
        polys.push_cell(&ids);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Merge vertices that are at the same position (within tolerance).
pub fn merge_coincident_vertices(mesh: &PolyData, tolerance: f64) -> PolyData {
    let n = mesh.points.len();
    let tol2 = tolerance * tolerance;
    let mut mapping = vec![0usize; n];
    let mut new_pts = Points::<f64>::new();
    let mut used = vec![false; n];

    for i in 0..n {
        if used[i] {
            continue;
        }
        let pi = mesh.points.get(i);
        let new_idx = new_pts.len();
        new_pts.push(pi);
        mapping[i] = new_idx;
        for j in i + 1..n {
            if used[j] {
                continue;
            }
            let pj = mesh.points.get(j);
            if (pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2) < tol2 {
                mapping[j] = new_idx;
                used[j] = true;
            }
        }
    }

    let mut polys = CellArray::new();
    for cell in mesh.polys.iter() {
        let ids: Vec<i64> = cell
            .iter()
            .map(|&pid| mapping[pid as usize] as i64)
            .collect();
        // Skip degenerate
        let unique: std::collections::HashSet<i64> = ids.iter().cloned().collect();
        if unique.len() >= 3 {
            polys.push_cell(&ids);
        }
    }

    let mut result = PolyData::new();
    result.points = new_pts;
    result.polys = polys;
    result
}

/// Create per-face vertex colors from cell data (by duplicating vertices).
pub fn cell_data_to_flat_vertex_colors(mesh: &PolyData, cell_array: &str) -> PolyData {
    let arr = match mesh.cell_data().get_array(cell_array) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let nc = arr.num_components();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut vert_data = Vec::new();
    let mut buf = vec![0.0f64; nc];

    for (ci, cell) in mesh.polys.iter().enumerate() {
        let base = pts.len() as i64;
        if ci < arr.num_tuples() {
            arr.tuple_as_f64(ci, &mut buf);
        }
        let ids: Vec<i64> = cell
            .iter()
            .enumerate()
            .map(|(i, &pid)| {
                pts.push(mesh.points.get(pid as usize));
                vert_data.extend_from_slice(&buf);
                base + i as i64
            })
            .collect();
        polys.push_cell(&ids);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            cell_array, vert_data, nc,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = cells_to_independent_vertices(&mesh);
        assert_eq!(result.points.len(), 6); // 2 tris × 3 verts each
    }
    #[test]
    fn merge() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.001, 0.0, 0.0],
                [1.001, 0.0, 0.0],
                [0.501, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = merge_coincident_vertices(&mesh, 0.01);
        assert!(result.points.len() <= 4);
    }
    #[test]
    fn flat_colors() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "color",
                vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                3,
            )));
        let result = cell_data_to_flat_vertex_colors(&mesh, "color");
        assert_eq!(result.points.len(), 6);
        assert!(result.point_data().get_array("color").is_some());
    }
}
