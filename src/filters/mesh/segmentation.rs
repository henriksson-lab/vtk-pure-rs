//! Mesh segmentation by curvature, normal clustering, and watershed.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Segment a mesh by face normal clustering using k-means.
///
/// Groups faces with similar normals into `k` segments.
pub fn segment_by_normals(mesh: &PolyData, k: usize) -> PolyData {
    let n_cells = mesh.polys.num_cells();
    if n_cells == 0 || k == 0 {
        return mesh.clone();
    }

    let normals: Vec<[f64; 3]> = mesh
        .polys
        .iter()
        .map(|cell| {
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
        })
        .collect();

    // K-means on normals
    let mut centroids: Vec<[f64; 3]> = (0..k).map(|i| normals[i * n_cells / k]).collect();
    let mut labels = vec![0usize; n_cells];

    for _ in 0..50 {
        // Assign
        for (ci, n) in normals.iter().enumerate() {
            let mut best = 0;
            let mut best_d = f64::MAX;
            for (ki, c) in centroids.iter().enumerate() {
                let d = (n[0] - c[0]).powi(2) + (n[1] - c[1]).powi(2) + (n[2] - c[2]).powi(2);
                if d < best_d {
                    best_d = d;
                    best = ki;
                }
            }
            labels[ci] = best;
        }
        // Update
        let mut new_c = vec![[0.0; 3]; k];
        let mut counts = vec![0usize; k];
        for (ci, &l) in labels.iter().enumerate() {
            for j in 0..3 {
                new_c[l][j] += normals[ci][j];
            }
            counts[l] += 1;
        }
        for ki in 0..k {
            if counts[ki] > 0 {
                let len =
                    (new_c[ki][0].powi(2) + new_c[ki][1].powi(2) + new_c[ki][2].powi(2)).sqrt();
                if len > 1e-15 {
                    for j in 0..3 {
                        new_c[ki][j] /= len;
                    }
                }
            }
        }
        centroids = new_c;
    }

    let seg_data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SegmentId",
            seg_data,
            1,
        )));
    result
}

/// Segment by connected components of similar dihedral angles.
///
/// Faces sharing an edge with dihedral angle below `threshold` (degrees)
/// are grouped together.
pub fn segment_by_dihedral_angle(mesh: &PolyData, threshold_degrees: f64) -> PolyData {
    let n_cells = mesh.polys.num_cells();
    if n_cells == 0 {
        return mesh.clone();
    }

    let cos_thresh = (threshold_degrees * std::f64::consts::PI / 180.0).cos();
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();

    let normals: Vec<[f64; 3]> = all_cells
        .iter()
        .map(|cell| {
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
        })
        .collect();

    // Build edge→cell adjacency
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

    // Connected components with dihedral threshold
    let mut labels = vec![usize::MAX; n_cells];
    let mut next_label = 0;
    for seed in 0..n_cells {
        if labels[seed] != usize::MAX {
            continue;
        }
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(seed);
        labels[seed] = next_label;
        while let Some(ci) = queue.pop_front() {
            let cell = &all_cells[ci];
            let nc = cell.len();
            for i in 0..nc {
                let a = cell[i] as usize;
                let b = cell[(i + 1) % nc] as usize;
                let edge = (a.min(b), a.max(b));
                if let Some(neighbors) = edge_cells.get(&edge) {
                    for &ni in neighbors {
                        if labels[ni] != usize::MAX {
                            continue;
                        }
                        let dot = normals[ci][0] * normals[ni][0]
                            + normals[ci][1] * normals[ni][1]
                            + normals[ci][2] * normals[ni][2];
                        if dot >= cos_thresh {
                            labels[ni] = next_label;
                            queue.push_back(ni);
                        }
                    }
                }
            }
        }
        next_label += 1;
    }

    let seg_data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SegmentId",
            seg_data,
            1,
        )));
    result
}

/// Count segments.
pub fn count_segments(mesh: &PolyData) -> usize {
    match mesh.cell_data().get_array("SegmentId") {
        Some(arr) => {
            let mut max_id = -1i64;
            let mut buf = [0.0f64];
            for i in 0..arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                max_id = max_id.max(buf[0] as i64);
            }
            if max_id >= 0 {
                (max_id + 1) as usize
            } else {
                0
            }
        }
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_segmentation() {
        // Cube-like: 6 face groups
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
        let result = segment_by_normals(&mesh, 6);
        assert!(result.cell_data().get_array("SegmentId").is_some());
        assert!(count_segments(&result) <= 6);
    }

    #[test]
    fn dihedral_segmentation() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 1.0, 1.0],
            ], // bent face
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = segment_by_dihedral_angle(&mesh, 10.0);
        assert!(result.cell_data().get_array("SegmentId").is_some());
    }

    #[test]
    fn flat_plane_one_segment() {
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
        let result = segment_by_dihedral_angle(&mesh, 10.0);
        assert_eq!(count_segments(&result), 1); // all coplanar
    }
}
