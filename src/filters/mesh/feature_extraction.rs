//! Extract geometric features: edges, corners, flat regions, creases.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Classify vertices as flat, edge, or corner based on dihedral angles.
///
/// 0 = flat (all adjacent dihedral angles < edge_threshold)
/// 1 = edge (one or more sharp edges but not a corner)
/// 2 = corner (multiple sharp edges meeting)
pub fn classify_vertices(mesh: &PolyData, edge_threshold_degrees: f64) -> PolyData {
    let n = mesh.points.len();
    let cos_thresh = (edge_threshold_degrees * std::f64::consts::PI / 180.0).cos();

    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let normals: Vec<[f64; 3]> = all_cells
        .iter()
        .map(|cell| face_normal(mesh, cell))
        .collect();

    // For each vertex, count how many sharp edges it touches
    let mut sharp_count = vec![0usize; n];
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();

    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    for (&(a, b), faces) in &edge_faces {
        if faces.len() == 2 {
            let dot = normals[faces[0]][0] * normals[faces[1]][0]
                + normals[faces[0]][1] * normals[faces[1]][1]
                + normals[faces[0]][2] * normals[faces[1]][2];
            if dot < cos_thresh {
                sharp_count[a] += 1;
                sharp_count[b] += 1;
            }
        } else if faces.len() == 1 {
            // Boundary edge counts as sharp
            sharp_count[a] += 1;
            sharp_count[b] += 1;
        }
    }

    let classification: Vec<f64> = sharp_count
        .iter()
        .map(|&c| {
            if c == 0 {
                0.0
            }
            // flat
            else if c <= 2 {
                1.0
            }
            // edge
            else {
                2.0
            } // corner
        })
        .collect();

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VertexClass",
            classification,
            1,
        )));
    result
}

/// Extract feature edges (sharp + boundary) as a line PolyData.
pub fn extract_feature_lines(mesh: &PolyData, edge_threshold_degrees: f64) -> PolyData {
    let cos_thresh = (edge_threshold_degrees * std::f64::consts::PI / 180.0).cos();
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let normals: Vec<[f64; 3]> = all_cells
        .iter()
        .map(|cell| face_normal(mesh, cell))
        .collect();

    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut feature_type = Vec::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for (&(a, b), faces) in &edge_faces {
        let is_feature = if faces.len() == 1 {
            true // boundary
        } else if faces.len() == 2 {
            let dot = normals[faces[0]][0] * normals[faces[1]][0]
                + normals[faces[0]][1] * normals[faces[1]][1]
                + normals[faces[0]][2] * normals[faces[1]][2];
            dot < cos_thresh
        } else {
            true // non-manifold
        };

        if !is_feature {
            continue;
        }

        let ia = *pt_map.entry(a).or_insert_with(|| {
            let i = pts.len();
            pts.push(mesh.points.get(a));
            i
        });
        let ib = *pt_map.entry(b).or_insert_with(|| {
            let i = pts.len();
            pts.push(mesh.points.get(b));
            i
        });
        lines.push_cell(&[ia as i64, ib as i64]);
        feature_type.push(if faces.len() == 1 {
            0.0
        } else if faces.len() == 2 {
            1.0
        } else {
            2.0
        });
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FeatureType",
            feature_type,
            1,
        )));
    result
}

/// Extract only flat regions (faces with all neighbors below angle threshold).
pub fn extract_flat_regions(mesh: &PolyData, threshold_degrees: f64) -> PolyData {
    let cos_thresh = (threshold_degrees * std::f64::consts::PI / 180.0).cos();
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let normals: Vec<[f64; 3]> = all_cells
        .iter()
        .map(|cell| face_normal(mesh, cell))
        .collect();

    let mut edge_cells: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_cells.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    // A face is "flat" if all its edge-neighbors have similar normals
    let mut is_flat = vec![true; all_cells.len()];
    for (_, faces) in &edge_cells {
        if faces.len() == 2 {
            let dot = normals[faces[0]][0] * normals[faces[1]][0]
                + normals[faces[0]][1] * normals[faces[1]][1]
                + normals[faces[0]][2] * normals[faces[1]][2];
            if dot < cos_thresh {
                is_flat[faces[0]] = false;
                is_flat[faces[1]] = false;
            }
        }
    }

    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for (ci, cell) in all_cells.iter().enumerate() {
        if !is_flat[ci] {
            continue;
        }
        let mut ids = Vec::new();
        for &pid in cell {
            let old = pid as usize;
            let idx = *pt_map.entry(old).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(old));
                i
            });
            ids.push(idx as i64);
        }
        polys.push_cell(&ids);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

fn face_normal(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let a = mesh.points.get(cell[0] as usize);
    let b = mesh.points.get(cell[1] as usize);
    let c = mesh.points.get(cell[2] as usize);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-15 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_cube_vertices() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            vec![
                [0, 2, 1],
                [0, 3, 2],
                [4, 5, 6],
                [4, 6, 7],
                [0, 1, 5],
                [0, 5, 4],
                [2, 3, 7],
                [2, 7, 6],
                [0, 4, 7],
                [0, 7, 3],
                [1, 2, 6],
                [1, 6, 5],
            ],
        );
        let result = classify_vertices(&mesh, 30.0);
        let arr = result.point_data().get_array("VertexClass").unwrap();
        let mut buf = [0.0f64];
        // Cube corners should be classified as corners (class 2)
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 2.0);
    }

    #[test]
    fn feature_lines() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        let lines = extract_feature_lines(&mesh, 30.0);
        assert!(lines.lines.num_cells() > 0);
    }

    #[test]
    fn flat_regions() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..3 {
            for x in 0..3 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..2 {
            for x in 0..2 {
                let bl = y * 3 + x;
                tris.push([bl, bl + 1, bl + 4]);
                tris.push([bl, bl + 4, bl + 3]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let flat = extract_flat_regions(&mesh, 10.0);
        assert_eq!(flat.polys.num_cells(), mesh.polys.num_cells()); // all flat
    }
}
