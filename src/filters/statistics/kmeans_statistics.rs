//! K-means clustering on tabular or point data.
//!
//! Assigns each row/point to one of K clusters by iterating between
//! assignment and centroid update steps.

use crate::data::{AnyDataArray, DataArray, PolyData, Table};

/// Result of K-means clustering.
#[derive(Debug, Clone)]
pub struct KmeansResult {
    /// Cluster assignment for each sample (0-based).
    pub labels: Vec<usize>,
    /// Cluster centroids (K x D).
    pub centroids: Vec<Vec<f64>>,
    /// Within-cluster sum of squared distances.
    pub inertia: f64,
    /// Number of iterations performed.
    pub iterations: usize,
}

/// Run K-means clustering on scalar columns of a Table.
///
/// Returns cluster labels, centroids, and inertia.
pub fn kmeans_table(table: &Table, k: usize, max_iter: usize) -> Option<KmeansResult> {
    let data = extract_matrix(table);
    if data.is_empty() || k == 0 {
        return None;
    }
    Some(kmeans_core(&data, k, max_iter))
}

/// Run K-means clustering on PolyData point positions.
///
/// Returns the mesh with a "ClusterId" point data array.
pub fn kmeans_points(mesh: &PolyData, k: usize, max_iter: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || k == 0 {
        return mesh.clone();
    }

    let data: Vec<Vec<f64>> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            vec![p[0], p[1], p[2]]
        })
        .collect();

    let result = kmeans_core(&data, k, max_iter);

    let mut out = mesh.clone();
    let labels: Vec<f64> = result.labels.iter().map(|&l| l as f64).collect();
    out.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ClusterId",
            labels,
            1,
        )));
    out
}

/// Run K-means on scalar columns of a Table and add a "ClusterId" column.
pub fn kmeans_table_labeled(table: &Table, k: usize, max_iter: usize) -> Table {
    let data = extract_matrix(table);
    if data.is_empty() || k == 0 {
        return table.clone();
    }

    let result = kmeans_core(&data, k, max_iter);
    let labels: Vec<f64> = result.labels.iter().map(|&l| l as f64).collect();

    let mut out = table.clone();
    out.add_column(AnyDataArray::F64(DataArray::from_vec(
        "ClusterId",
        labels,
        1,
    )));
    out
}

fn kmeans_core(data: &[Vec<f64>], k: usize, max_iter: usize) -> KmeansResult {
    let n = data.len();
    let d = data[0].len();
    let k = k.min(n);

    let mut centroids: Vec<Vec<f64>> = data[..k].to_vec();
    let mut labels = vec![usize::MAX; n];
    let mut iterations = 0;
    let tolerance = 0.01f64;

    for iter in 0..max_iter.max(1) {
        iterations = iter + 1;

        // Assignment step
        let mut membership_changes = 0usize;
        for (i, row) in data.iter().enumerate() {
            let mut best_c = 0;
            let mut best_d = f64::MAX;
            for (ci, c) in centroids.iter().enumerate() {
                let d = sq_dist(row, c);
                if d < best_d {
                    best_d = d;
                    best_c = ci;
                }
            }
            if labels[i] != best_c {
                labels[i] = best_c;
                membership_changes += 1;
            }
        }

        // Update step
        let mut new_centroids = vec![vec![0.0; d]; k];
        let mut counts = vec![0usize; k];
        for (i, row) in data.iter().enumerate() {
            let c = labels[i];
            counts[c] += 1;
            for j in 0..d {
                new_centroids[c][j] += row[j];
            }
        }
        for ci in 0..k {
            if counts[ci] > 0 {
                for j in 0..d {
                    new_centroids[ci][j] /= counts[ci] as f64;
                }
            } else {
                new_centroids[ci].clone_from(&centroids[ci]);
            }
        }
        centroids = new_centroids;

        if (membership_changes as f64 / n as f64) < tolerance {
            break;
        }
    }

    // Compute inertia
    let mut inertia = 0.0;
    for (i, row) in data.iter().enumerate() {
        inertia += sq_dist(row, &centroids[labels[i]]);
    }

    KmeansResult {
        labels,
        centroids,
        inertia,
        iterations,
    }
}

fn sq_dist(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum()
}

fn extract_matrix(table: &Table) -> Vec<Vec<f64>> {
    let mut cols: Vec<Vec<f64>> = Vec::new();
    for col in table.columns() {
        if col.num_components() != 1 {
            continue;
        }
        let n = col.num_tuples();
        let mut values = Vec::with_capacity(n);
        let mut buf = [0.0f64];
        for i in 0..n {
            col.tuple_as_f64(i, &mut buf);
            values.push(buf[0]);
        }
        cols.push(values);
    }
    if cols.is_empty() {
        return Vec::new();
    }
    let n = cols[0].len();
    (0..n)
        .map(|i| cols.iter().map(|c| c[i]).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_clusters() {
        let table = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![0.0, 0.1, 0.2, 10.0, 10.1, 10.2],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![0.0, 0.1, -0.1, 0.0, 0.1, -0.1],
                1,
            )));

        let result = kmeans_table(&table, 2, 100).unwrap();
        assert_eq!(result.labels.len(), 6);
        assert_eq!(result.centroids.len(), 2);
        // First 3 should be in same cluster, last 3 in another
        assert_eq!(result.labels[0], result.labels[1]);
        assert_eq!(result.labels[0], result.labels[2]);
        assert_eq!(result.labels[3], result.labels[4]);
        assert_ne!(result.labels[0], result.labels[3]);
    }

    #[test]
    fn kmeans_on_points() {
        let mut mesh = PolyData::new();
        mesh.points = crate::data::Points::from(vec![
            [0.0, 0.0, 0.0],
            [0.1, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.1, 0.0, 0.0],
        ]);
        let result = kmeans_points(&mesh, 2, 50);
        assert!(result.point_data().get_array("ClusterId").is_some());
    }

    #[test]
    fn labeled_table() {
        let table = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![0.0, 0.1, 5.0, 5.1],
            1,
        )));
        let result = kmeans_table_labeled(&table, 2, 50);
        assert!(result.column_by_name("ClusterId").is_some());
    }

    #[test]
    fn single_cluster() {
        let table = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0, 2.0, 3.0],
            1,
        )));
        let result = kmeans_table(&table, 1, 50).unwrap();
        assert!(result.labels.iter().all(|&l| l == 0));
    }
}
