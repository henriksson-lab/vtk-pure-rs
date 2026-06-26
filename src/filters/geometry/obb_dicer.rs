//! Spatial decomposition using oriented bounding boxes.
//!
//! Recursively subdivides the OBB of a PolyData mesh along the longest axis
//! until each partition has fewer than `max_points` points.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Dice a PolyData into regions using recursive OBB subdivision.
///
/// Splits along the longest axis of the oriented bounding box, using point
/// positions to determine which side each point belongs to. Recursion stops
/// when a region has at most `max_points` points.
///
/// Adds a "vtkOBBDicer_GroupIds" point data array to the output.
pub fn obb_dicer(input: &PolyData, max_points: usize) -> PolyData {
    let max_points = max_points.max(1);
    let num_points = input.points.len();
    if num_points == 0 {
        return input.clone();
    }

    let points: Vec<[f64; 3]> = (0..num_points).map(|i| input.points.get(i)).collect();

    // Assign region IDs via recursive splitting
    let mut group_ids = vec![0i32; num_points];
    let indices: Vec<usize> = (0..num_points).collect();
    let mut next_region = 0i32;
    recursive_split(
        &points,
        &indices,
        max_points,
        &mut group_ids,
        &mut next_region,
    );

    let group_f64: Vec<f64> = group_ids.iter().map(|&r| r as f64).collect();
    let mut output = input.clone();
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "vtkOBBDicer_GroupIds",
            group_f64,
            1,
        )));
    output
}

fn recursive_split(
    points: &[[f64; 3]],
    indices: &[usize],
    max_points: usize,
    group_ids: &mut [i32],
    next_region: &mut i32,
) {
    if indices.len() <= max_points || indices.is_empty() {
        let id = *next_region;
        *next_region += 1;
        for &idx in indices {
            group_ids[idx] = id;
        }
        return;
    }

    // Compute mean point for this subset
    let mut mean = [0.0f64; 3];
    for &idx in indices {
        let p = points[idx];
        mean[0] += p[0];
        mean[1] += p[1];
        mean[2] += p[2];
    }
    let n = indices.len() as f64;
    mean[0] /= n;
    mean[1] /= n;
    mean[2] /= n;

    // Compute covariance matrix for OBB principal axis
    let mut cov = [[0.0f64; 3]; 3];
    for &idx in indices {
        let p = points[idx];
        let d = [p[0] - mean[0], p[1] - mean[1], p[2] - mean[2]];
        for i in 0..3 {
            for j in 0..3 {
                cov[i][j] += d[i] * d[j];
            }
        }
    }

    // Find the longest axis using power iteration (simple approximation)
    let axis = dominant_eigenvector(&cov);

    // Project points onto the axis and split at the median
    let mut projections: Vec<(f64, usize)> = indices
        .iter()
        .map(|&idx| {
            let p = points[idx];
            let proj = (p[0] - mean[0]) * axis[0]
                + (p[1] - mean[1]) * axis[1]
                + (p[2] - mean[2]) * axis[2];
            (proj, idx)
        })
        .collect();

    projections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mid = projections.len() / 2;
    let left: Vec<usize> = projections[..mid].iter().map(|&(_, idx)| idx).collect();
    let right: Vec<usize> = projections[mid..].iter().map(|&(_, idx)| idx).collect();

    recursive_split(points, &left, max_points, group_ids, next_region);
    recursive_split(points, &right, max_points, group_ids, next_region);
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
        // Create 18 points spread along x-axis
        for i in 0..6 {
            let x = i as f64 * 2.0;
            let base = (i * 3) as i64;
            pd.points.push([x, 0.0, 0.0]);
            pd.points.push([x + 1.0, 0.0, 0.0]);
            pd.points.push([x + 0.5, 1.0, 0.0]);
            pd.polys.push_cell(&[base, base + 1, base + 2]);
        }

        let result = obb_dicer(&pd, 2);
        let arr = result
            .point_data()
            .get_array("vtkOBBDicer_GroupIds")
            .unwrap();
        assert_eq!(arr.num_tuples(), 18);

        // Should have at least 9 regions (18 points / 2 max each)
        let mut ids = Vec::new();
        let mut buf = [0.0f64];
        for i in 0..18 {
            arr.tuple_as_f64(i, &mut buf);
            ids.push(buf[0] as i32);
        }
        ids.sort();
        ids.dedup();
        assert!(ids.len() >= 9);
    }

    #[test]
    fn single_cell() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = obb_dicer(&pd, 10);
        let arr = result
            .point_data()
            .get_array("vtkOBBDicer_GroupIds")
            .unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
