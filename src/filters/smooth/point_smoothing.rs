//! Geometric and scalar point cloud smoothing.
//!
//! Smooths point positions and/or scalar values by averaging with
//! k-nearest neighbors.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Smooth point cloud positions by averaging with k-nearest neighbors.
///
/// Each point is moved toward the centroid of its k nearest neighbors
/// by the given factor (0 = no smoothing, 1 = full average).
pub fn smooth_point_positions(
    mesh: &PolyData,
    k: usize,
    factor: f64,
    iterations: usize,
) -> PolyData {
    let n = mesh.points.len();
    if n < 2 || k == 0 {
        return mesh.clone();
    }

    let mut positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    for _ in 0..iterations {
        let mut new_pos = positions.clone();
        for i in 0..n {
            let neighbors = find_knn(&positions, i, k);
            let mut avg = [0.0; 3];
            for &ni in &neighbors {
                for c in 0..3 {
                    avg[c] += positions[ni][c];
                }
            }
            let nk = neighbors.len() as f64;
            if nk > 0.0 {
                for c in 0..3 {
                    avg[c] /= nk;
                    new_pos[i][c] = positions[i][c] * (1.0 - factor) + avg[c] * factor;
                }
            }
        }
        positions = new_pos;
    }

    let mut result = mesh.clone();
    result.points = Points::from(positions);
    result
}

/// Smooth scalar point data by averaging with k-nearest neighbors.
pub fn smooth_point_scalars(
    mesh: &PolyData,
    array_name: &str,
    k: usize,
    factor: f64,
    iterations: usize,
) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };

    let positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let mut values = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values.push(buf[0]);
    }

    for _ in 0..iterations {
        let mut new_vals = values.clone();
        for i in 0..n {
            let neighbors = find_knn(&positions, i, k);
            let mut avg = 0.0;
            for &ni in &neighbors {
                avg += values[ni];
            }
            let nk = neighbors.len() as f64;
            if nk > 0.0 {
                new_vals[i] = values[i] * (1.0 - factor) + (avg / nk) * factor;
            }
        }
        values = new_vals;
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, 1,
        )));
    result
}

fn find_knn(points: &[[f64; 3]], query_idx: usize, k: usize) -> Vec<usize> {
    let q = points[query_idx];
    let mut dists: Vec<(usize, f64)> = points
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != query_idx)
        .map(|(i, p)| {
            let d = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
            (i, d)
        })
        .collect();
    dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    dists.iter().take(k).map(|(i, _)| *i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooth_positions() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 0.1, 0.0],
            [0.5, -0.1, 0.0],
            [2.0, 0.0, 0.0],
        ]);
        let result = smooth_point_positions(&mesh, 3, 0.5, 2);
        assert_eq!(result.points.len(), 5);
        // Points should be closer together after smoothing
    }

    #[test]
    fn smooth_scalars() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 100.0, 0.0],
                1,
            )));

        let result = smooth_point_scalars(&mesh, "val", 2, 0.5, 1);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert!(buf[0] < 100.0); // should be smoothed down
        assert!(buf[0] > 0.0);
    }

    #[test]
    fn empty_mesh() {
        let mesh = PolyData::new();
        let result = smooth_point_positions(&mesh, 3, 0.5, 1);
        assert_eq!(result.points.len(), 0);
    }
}
