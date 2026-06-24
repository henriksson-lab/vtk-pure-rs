//! Distributed data decomposition (D3-style).
//!
//! Splits a dataset into spatially balanced partitions using k-d tree
//! recursive bisection, suitable for distributed-memory parallelism.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Recursively bisect a point cloud into 2^depth partitions.
///
/// Uses alternating axis splits (like a k-d tree) for spatial balance.
/// Returns partitions with "PartitionId" point data.
pub fn d3_decompose_points(mesh: &PolyData, depth: usize) -> Vec<PolyData> {
    let n = mesh.points.len();
    if n == 0 {
        return Vec::new();
    }

    let indices: Vec<usize> = (0..n).collect();
    let positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    let mut partitions: Vec<Vec<usize>> = vec![indices];

    for level in 0..depth {
        let axis = level % 3;
        let mut new_partitions = Vec::new();

        for part in &partitions {
            if part.len() <= 1 {
                new_partitions.push(part.clone());
                continue;
            }

            let mut sorted = part.clone();
            sorted.sort_by(|&a, &b| {
                positions[a][axis]
                    .partial_cmp(&positions[b][axis])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let mid = sorted.len() / 2;
            new_partitions.push(sorted[..mid].to_vec());
            new_partitions.push(sorted[mid..].to_vec());
        }

        partitions = new_partitions;
    }

    partitions
        .iter()
        .enumerate()
        .map(|(pid, indices)| {
            let mut pts = Points::<f64>::new();
            for &idx in indices {
                pts.push(positions[idx]);
            }
            let ids = vec![pid as f64; indices.len()];
            let mut part = PolyData::new();
            part.points = pts;
            part.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    "PartitionId",
                    ids,
                    1,
                )));
            part
        })
        .collect()
}

/// Decompose a triangle mesh into balanced partitions.
pub fn d3_decompose_cells(mesh: &PolyData, depth: usize) -> Vec<PolyData> {
    let n_cells = mesh.polys.num_cells();
    if n_cells == 0 {
        return Vec::new();
    }

    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();

    // Compute cell centroids
    let centroids: Vec<[f64; 3]> = all_cells
        .iter()
        .map(|cell| {
            let mut c = [0.0; 3];
            for &pid in cell {
                let p = mesh.points.get(pid as usize);
                for j in 0..3 {
                    c[j] += p[j];
                }
            }
            let n = cell.len() as f64;
            [c[0] / n, c[1] / n, c[2] / n]
        })
        .collect();

    let cell_indices: Vec<usize> = (0..n_cells).collect();
    let mut partitions = vec![cell_indices];

    for level in 0..depth {
        let axis = level % 3;
        let mut new_parts = Vec::new();
        for part in &partitions {
            if part.len() <= 1 {
                new_parts.push(part.clone());
                continue;
            }
            let mut sorted = part.clone();
            sorted.sort_by(|&a, &b| {
                centroids[a][axis]
                    .partial_cmp(&centroids[b][axis])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let mid = sorted.len() / 2;
            new_parts.push(sorted[..mid].to_vec());
            new_parts.push(sorted[mid..].to_vec());
        }
        partitions = new_parts;
    }

    partitions
        .iter()
        .map(|cell_idxs| {
            let mut pts = Points::<f64>::new();
            let mut polys = CellArray::new();
            let mut pt_map: std::collections::HashMap<usize, usize> =
                std::collections::HashMap::new();
            for &ci in cell_idxs {
                let cell = &all_cells[ci];
                let mut new_ids = Vec::new();
                for &pid in cell {
                    let old = pid as usize;
                    let new_idx = *pt_map.entry(old).or_insert_with(|| {
                        let idx = pts.len();
                        pts.push(mesh.points.get(old));
                        idx
                    });
                    new_ids.push(new_idx as i64);
                }
                polys.push_cell(&new_ids);
            }
            let mut part = PolyData::new();
            part.points = pts;
            part.polys = polys;
            part
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompose_points_4() {
        let mesh = PolyData::from_points(
            (0..100)
                .map(|i| [i as f64, (i * 7 % 50) as f64, 0.0])
                .collect::<Vec<_>>(),
        );
        let parts = d3_decompose_points(&mesh, 2);
        assert_eq!(parts.len(), 4);
        let total: usize = parts.iter().map(|p| p.points.len()).sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn decompose_cells_2() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for i in 0..10 {
            for j in 0..10 {
                pts.push([i as f64, j as f64, 0.0]);
            }
        }
        for i in 0..9 {
            for j in 0..9 {
                let bl = i * 10 + j;
                tris.push([bl, bl + 1, bl + 11]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let parts = d3_decompose_cells(&mesh, 1);
        assert_eq!(parts.len(), 2);
        let total: usize = parts.iter().map(|p| p.polys.num_cells()).sum();
        assert_eq!(total, 81);
    }

    #[test]
    fn single_point() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0]]);
        let parts = d3_decompose_points(&mesh, 3);
        // Should produce partitions, some may be empty
        assert!(parts.len() >= 1);
    }
}
