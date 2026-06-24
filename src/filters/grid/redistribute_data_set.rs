//! Load balancing / redistribution of datasets across partitions.
//!
//! Splits a mesh into balanced partitions using spatial sorting.

use crate::data::{CellArray, Points, PolyData};

/// Redistribute a PolyData into N roughly equal-sized partitions.
///
/// Uses spatial sorting along the longest axis to produce spatially
/// coherent partitions with balanced point counts.
pub fn redistribute_points(mesh: &PolyData, n_partitions: usize) -> Vec<PolyData> {
    let n = mesh.points.len();
    if n == 0 || n_partitions == 0 {
        return Vec::new();
    }

    // Find longest axis
    let mut min = mesh.points.get(0);
    let mut max = min;
    for i in 1..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            min[j] = min[j].min(p[j]);
            max[j] = max[j].max(p[j]);
        }
    }
    let extents = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let axis = if extents[0] >= extents[1] && extents[0] >= extents[2] {
        0
    } else if extents[1] >= extents[2] {
        1
    } else {
        2
    };

    // Sort point indices by position along longest axis
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&a, &b| {
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        pa[axis]
            .partial_cmp(&pb[axis])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Split into partitions
    let chunk_size = (n + n_partitions - 1) / n_partitions;
    let mut partitions = Vec::new();

    for chunk in indices.chunks(chunk_size) {
        let mut pts = Points::<f64>::new();
        for &idx in chunk {
            pts.push(mesh.points.get(idx));
        }
        let mut part = PolyData::new();
        part.points = pts;
        partitions.push(part);
    }

    partitions
}

/// Redistribute a triangle mesh into N partitions by cell centroids.
///
/// Returns partitions with cells, preserving connectivity within each partition.
pub fn redistribute_cells(mesh: &PolyData, n_partitions: usize) -> Vec<PolyData> {
    let n_cells = mesh.polys.num_cells();
    if n_cells == 0 || n_partitions == 0 {
        return Vec::new();
    }

    // Compute cell centroids
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut centroids: Vec<(usize, f64)> = Vec::with_capacity(n_cells);

    let _min_v = f64::MAX;
    let _max_v = f64::MIN;
    // Find longest axis from bounding box
    let mut bb_min = mesh.points.get(0);
    let mut bb_max = bb_min;
    for i in 1..mesh.points.len() {
        let p = mesh.points.get(i);
        for j in 0..3 {
            bb_min[j] = bb_min[j].min(p[j]);
            bb_max[j] = bb_max[j].max(p[j]);
        }
    }
    let ext = [
        bb_max[0] - bb_min[0],
        bb_max[1] - bb_min[1],
        bb_max[2] - bb_min[2],
    ];
    let axis = if ext[0] >= ext[1] && ext[0] >= ext[2] {
        0
    } else if ext[1] >= ext[2] {
        1
    } else {
        2
    };

    for (ci, cell) in all_cells.iter().enumerate() {
        let mut c = 0.0;
        for &pid in cell {
            c += mesh.points.get(pid as usize)[axis];
        }
        let val = c / cell.len() as f64;
        centroids.push((ci, val));
    }
    centroids.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let chunk_size = (n_cells + n_partitions - 1) / n_partitions;
    let mut partitions = Vec::new();

    for chunk in centroids.chunks(chunk_size) {
        let mut pts = Points::<f64>::new();
        let mut polys = CellArray::new();
        let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

        for &(ci, _) in chunk {
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
        partitions.push(part);
    }

    partitions
}

/// Compute load balance ratio (min/max partition size).
pub fn balance_ratio(partitions: &[PolyData]) -> f64 {
    if partitions.is_empty() {
        return 1.0;
    }
    let sizes: Vec<usize> = partitions.iter().map(|p| p.points.len()).collect();
    let min = *sizes.iter().min().unwrap() as f64;
    let max = *sizes.iter().max().unwrap() as f64;
    if max < 1.0 {
        1.0
    } else {
        min / max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redistribute_points_balanced() {
        let mesh =
            PolyData::from_points((0..100).map(|i| [i as f64, 0.0, 0.0]).collect::<Vec<_>>());
        let parts = redistribute_points(&mesh, 4);
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0].points.len(), 25);
        assert!(balance_ratio(&parts) > 0.99);
    }

    #[test]
    fn redistribute_cells_test() {
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
        let parts = redistribute_cells(&mesh, 3);
        assert_eq!(parts.len(), 3);
        let total: usize = parts.iter().map(|p| p.polys.num_cells()).sum();
        assert_eq!(total, 81);
    }

    #[test]
    fn empty() {
        assert!(redistribute_points(&PolyData::new(), 4).is_empty());
    }
}
