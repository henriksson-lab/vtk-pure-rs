//! Spatial decomposition using oriented bounding boxes.
//!
//! Recursively subdivides the OBB of a PolyData mesh along the longest axis
//! until each partition has fewer than `max_cells` cells.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Dice a PolyData into regions using recursive OBB subdivision.
///
/// Splits along the longest axis of the oriented bounding box, using cell
/// centroids to determine which side each cell belongs to. Recursion stops
/// when a region has at most `max_cells` cells.
///
/// Adds a "RegionId" cell data array to the output.
pub fn obb_dicer(input: &PolyData, max_cells: usize) -> PolyData {
    let max_cells = max_cells.max(1);
    let num_cells = input.polys.num_cells();
    if num_cells == 0 {
        return input.clone();
    }

    // Compute centroids for all cells
    let mut centroids: Vec<[f64; 3]> = Vec::with_capacity(num_cells);
    for ci in 0..num_cells {
        let cell = input.polys.cell(ci);
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &id in cell {
            let p = input.points.get(id as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = cell.len() as f64;
        centroids.push([cx / n, cy / n, cz / n]);
    }

    // Assign region IDs via recursive splitting
    let mut region_ids = vec![0i32; num_cells];
    let indices: Vec<usize> = (0..num_cells).collect();
    let mut next_region = 0i32;
    recursive_split(
        &centroids,
        &indices,
        max_cells,
        &mut region_ids,
        &mut next_region,
    );

    let region_f64: Vec<f64> = region_ids.iter().map(|&r| r as f64).collect();
    let mut output = input.clone();
    output
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "RegionId", region_f64, 1,
        )));
    output
}

fn recursive_split(
    centroids: &[[f64; 3]],
    indices: &[usize],
    max_cells: usize,
    region_ids: &mut [i32],
    next_region: &mut i32,
) {
    if indices.len() <= max_cells || indices.is_empty() {
        let id = *next_region;
        *next_region += 1;
        for &idx in indices {
            region_ids[idx] = id;
        }
        return;
    }

    // Compute mean centroid for this subset
    let mut mean = [0.0f64; 3];
    for &idx in indices {
        let c = centroids[idx];
        mean[0] += c[0];
        mean[1] += c[1];
        mean[2] += c[2];
    }
    let n = indices.len() as f64;
    mean[0] /= n;
    mean[1] /= n;
    mean[2] /= n;

    // Compute covariance matrix for OBB principal axis
    let mut cov = [[0.0f64; 3]; 3];
    for &idx in indices {
        let c = centroids[idx];
        let d = [c[0] - mean[0], c[1] - mean[1], c[2] - mean[2]];
        for i in 0..3 {
            for j in 0..3 {
                cov[i][j] += d[i] * d[j];
            }
        }
    }

    // Find the longest axis using power iteration (simple approximation)
    let axis = dominant_eigenvector(&cov);

    // Project centroids onto the axis and split at the median
    let mut projections: Vec<(f64, usize)> = indices
        .iter()
        .map(|&idx| {
            let c = centroids[idx];
            let proj = (c[0] - mean[0]) * axis[0]
                + (c[1] - mean[1]) * axis[1]
                + (c[2] - mean[2]) * axis[2];
            (proj, idx)
        })
        .collect();

    projections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mid = projections.len() / 2;
    let left: Vec<usize> = projections[..mid].iter().map(|&(_, idx)| idx).collect();
    let right: Vec<usize> = projections[mid..].iter().map(|&(_, idx)| idx).collect();

    recursive_split(centroids, &left, max_cells, region_ids, next_region);
    recursive_split(centroids, &right, max_cells, region_ids, next_region);
}

/// Simple power iteration to find the dominant eigenvector of a 3x3 symmetric matrix.
fn dominant_eigenvector(m: &[[f64; 3]; 3]) -> [f64; 3] {
    let mut v = [1.0, 0.0, 0.0];
    for _ in 0..20 {
        let nv = [
            m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
            m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
            m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
        ];
        let len = (nv[0] * nv[0] + nv[1] * nv[1] + nv[2] * nv[2]).sqrt();
        if len < 1e-15 {
            return [1.0, 0.0, 0.0]; // degenerate
        }
        v = [nv[0] / len, nv[1] / len, nv[2] / len];
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dice_small_mesh() {
        let mut pd = PolyData::new();
        // Create 6 triangles spread along x-axis
        for i in 0..6 {
            let x = i as f64 * 2.0;
            let base = (i * 3) as i64;
            pd.points.push([x, 0.0, 0.0]);
            pd.points.push([x + 1.0, 0.0, 0.0]);
            pd.points.push([x + 0.5, 1.0, 0.0]);
            pd.polys.push_cell(&[base, base + 1, base + 2]);
        }

        let result = obb_dicer(&pd, 2);
        let arr = result.cell_data().get_array("RegionId").unwrap();
        assert_eq!(arr.num_tuples(), 6);

        // Should have at least 3 regions (6 cells / 2 max each)
        let mut ids = Vec::new();
        let mut buf = [0.0f64];
        for i in 0..6 {
            arr.tuple_as_f64(i, &mut buf);
            ids.push(buf[0] as i32);
        }
        ids.sort();
        ids.dedup();
        assert!(ids.len() >= 3);
    }

    #[test]
    fn single_cell() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = obb_dicer(&pd, 10);
        let arr = result.cell_data().get_array("RegionId").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
