//! Approximate convex decomposition of meshes.
//!
//! Splits a non-convex mesh into approximately convex parts using
//! concavity-based recursive splitting.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Decompose a mesh into approximately convex parts.
///
/// Uses recursive bisection along planes where concavity is highest.
/// Returns a mesh with "ConvexPartId" cell data.
pub fn convex_decompose_mesh(mesh: &PolyData, max_concavity: f64, max_parts: usize) -> PolyData {
    let n_cells = mesh.polys.num_cells();
    if n_cells == 0 {
        return mesh.clone();
    }

    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
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
            let k = cell.len() as f64;
            [c[0] / k, c[1] / k, c[2] / k]
        })
        .collect();

    let mut labels = vec![0usize; n_cells];
    let mut next_label = 1;
    let mut parts: Vec<Vec<usize>> = vec![(0..n_cells).collect()];

    for _ in 0..max_parts.saturating_sub(1) {
        // Find the part with highest concavity
        let mut worst_part = 0;
        let mut worst_concavity = 0.0f64;

        for (pi, part) in parts.iter().enumerate() {
            if part.len() < 2 {
                continue;
            }
            let conc = estimate_concavity(mesh, &centroids, part);
            if conc > worst_concavity {
                worst_concavity = conc;
                worst_part = pi;
            }
        }

        if worst_concavity < max_concavity {
            break;
        }

        // Split the worst part along its longest axis
        let part = parts[worst_part].clone();
        let (left, right) = split_part(&centroids, &part);

        if left.is_empty() || right.is_empty() {
            break;
        }

        // Update labels
        for &ci in &right {
            labels[ci] = next_label;
        }
        next_label += 1;

        parts[worst_part] = left;
        parts.push(right);
    }

    let label_data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ConvexPartId",
            label_data,
            1,
        )));
    result
}

/// Count the number of convex parts.
pub fn count_convex_parts(mesh: &PolyData) -> usize {
    match mesh.cell_data().get_array("ConvexPartId") {
        Some(arr) => {
            let mut max_id = 0i64;
            let mut buf = [0.0f64];
            for i in 0..arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                max_id = max_id.max(buf[0] as i64);
            }
            (max_id + 1) as usize
        }
        None => 1,
    }
}

fn estimate_concavity(mesh: &PolyData, centroids: &[[f64; 3]], part: &[usize]) -> f64 {
    if part.len() < 2 {
        return 0.0;
    }
    // Compute convex hull diameter and check if points deviate from convex shape
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for &ci in part {
        cx += centroids[ci][0];
        cy += centroids[ci][1];
        cz += centroids[ci][2];
    }
    let k = part.len() as f64;
    cx /= k;
    cy /= k;
    cz /= k;

    let mut max_dist = 0.0f64;
    for &ci in part {
        let d = (centroids[ci][0] - cx).powi(2)
            + (centroids[ci][1] - cy).powi(2)
            + (centroids[ci][2] - cz).powi(2);
        max_dist = max_dist.max(d);
    }
    let diameter = max_dist.sqrt();

    // Rough concavity: variance of distances from centroid
    let mut var = 0.0;
    for &ci in part {
        let d = ((centroids[ci][0] - cx).powi(2)
            + (centroids[ci][1] - cy).powi(2)
            + (centroids[ci][2] - cz).powi(2))
        .sqrt();
        var += (d - diameter * 0.5).powi(2);
    }
    (var / k).sqrt() / diameter.max(1e-15)
}

fn split_part(centroids: &[[f64; 3]], part: &[usize]) -> (Vec<usize>, Vec<usize>) {
    if part.len() < 2 {
        return (part.to_vec(), Vec::new());
    }

    // Find longest axis
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    for &ci in part {
        for j in 0..3 {
            min[j] = min[j].min(centroids[ci][j]);
            max[j] = max[j].max(centroids[ci][j]);
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

    // Split at median
    let mut sorted = part.to_vec();
    sorted.sort_by(|&a, &b| {
        centroids[a][axis]
            .partial_cmp(&centroids[b][axis])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mid = sorted.len() / 2;
    (sorted[..mid].to_vec(), sorted[mid..].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompose_l_shape() {
        // L-shaped mesh (non-convex)
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3], [4, 5, 6], [4, 6, 7]],
        );
        let result = convex_decompose_mesh(&mesh, 0.01, 4);
        assert!(result.cell_data().get_array("ConvexPartId").is_some());
    }

    #[test]
    fn single_triangle() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = convex_decompose_mesh(&mesh, 0.1, 4);
        assert_eq!(count_convex_parts(&result), 1);
    }

    #[test]
    fn max_parts_limit() {
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
        let result = convex_decompose_mesh(&mesh, 0.001, 3);
        assert!(count_convex_parts(&result) <= 3);
    }
}
