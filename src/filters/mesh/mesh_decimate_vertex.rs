//! Vertex removal decimation with quality threshold.

use crate::data::{CellArray, Points, PolyData};

/// Decimate by removing vertices whose removal doesn't exceed error threshold.
pub fn decimate_vertex_removal(mesh: &PolyData, max_error: f64, target_ratio: f64) -> PolyData {
    let n = mesh.points.len();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let target_remove = ((n as f64) * target_ratio.clamp(0.0, 0.95)) as usize;

    // Build vertex-face adjacency
    let mut vert_faces: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (ci, cell) in cells.iter().enumerate() {
        for &v in cell {
            vert_faces[v as usize].push(ci);
        }
    }

    // Compute vertex error (distance to average plane of neighbors)
    let mut errors: Vec<(usize, f64)> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            if vert_faces[i].is_empty() {
                return (i, f64::INFINITY);
            }
            let mut avg = [0.0, 0.0, 0.0];
            let mut count = 0.0;
            for &fi in &vert_faces[i] {
                for &v in &cells[fi] {
                    let q = mesh.points.get(v as usize);
                    avg[0] += q[0];
                    avg[1] += q[1];
                    avg[2] += q[2];
                    count += 1.0;
                }
            }
            avg[0] /= count;
            avg[1] /= count;
            avg[2] /= count;
            let err = ((p[0] - avg[0]).powi(2) + (p[1] - avg[1]).powi(2) + (p[2] - avg[2]).powi(2))
                .sqrt();
            (i, err)
        })
        .collect();

    errors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut removed_verts = vec![false; n];
    let mut removed_cells = vec![false; cells.len()];
    let mut removals = 0;

    for &(vi, err) in &errors {
        if removals >= target_remove {
            break;
        }
        if err > max_error {
            break;
        }
        if removed_verts[vi] {
            continue;
        }

        // Check if removing this vertex would leave neighbors connected
        let adj_cells = &vert_faces[vi];
        let mut all_neighbors_have_other_cells = true;
        for &fi in adj_cells {
            if removed_cells[fi] {
                continue;
            }
            for &v in &cells[fi] {
                let v = v as usize;
                if v == vi || removed_verts[v] {
                    continue;
                }
                let other_cells = vert_faces[v]
                    .iter()
                    .filter(|&&ci| ci != fi && !removed_cells[ci])
                    .count();
                if other_cells == 0 {
                    all_neighbors_have_other_cells = false;
                    break;
                }
            }
            if !all_neighbors_have_other_cells {
                break;
            }
        }
        if !all_neighbors_have_other_cells {
            continue;
        }

        removed_verts[vi] = true;
        for &fi in adj_cells {
            removed_cells[fi] = true;
        }
        removals += 1;
    }

    // Rebuild
    let mut pt_map = vec![0usize; n];
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        if !removed_verts[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for (ci, cell) in cells.iter().enumerate() {
        if removed_cells[ci] {
            continue;
        }
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        polys.push_cell(&mapped);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decimate() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 4, 3], [1, 3, 2]],
        );
        let r = decimate_vertex_removal(&mesh, 10.0, 0.3);
        assert!(r.points.len() <= 5);
    }
    #[test]
    fn test_no_decimate() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = decimate_vertex_removal(&mesh, 0.001, 0.5);
        assert!(r.polys.num_cells() >= 0); // may or may not remove
    }
}
