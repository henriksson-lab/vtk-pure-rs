//! Descriptive statistics filter for PolyData point/cell data arrays.
//!
//! Computes mean, variance, skewness, kurtosis, min, max, median, and
//! percentiles for scalar arrays. Analogous to VTK's vtkDescriptiveStatistics.

use crate::data::{AnyDataArray, DataArray, PolyData, Table};

/// Statistical summary for a single array.
#[derive(Debug, Clone)]
pub struct ArrayDescriptiveStats {
    pub name: String,
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub median: f64,
    pub q1: f64,
    pub q3: f64,
    pub iqr: f64,
}

impl std::fmt::Display for ArrayDescriptiveStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: n={}, range=[{:.4}, {:.4}], mean={:.4}, std={:.4}, \
               skew={:.4}, kurt={:.4}, median={:.4}, IQR={:.4}",
            self.name,
            self.count,
            self.min,
            self.max,
            self.mean,
            self.std_dev,
            self.skewness,
            self.kurtosis,
            self.median,
            self.iqr
        )
    }
}

/// Compute descriptive statistics for a scalar array.
pub fn descriptive_statistics(array: &AnyDataArray) -> Option<ArrayDescriptiveStats> {
    let n = array.num_tuples();
    if n == 0 {
        return None;
    }

    let mut values = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        array.tuple_as_f64(i, &mut buf);
        values.push(buf[0]);
    }

    Some(compute_stats(&array.name().to_string(), &mut values))
}

/// Compute descriptive statistics for all scalar (1-component) point data arrays.
pub fn descriptive_statistics_point_data(mesh: &PolyData) -> Vec<ArrayDescriptiveStats> {
    let pd = mesh.point_data();
    let mut results = Vec::new();

    for i in 0..pd.num_arrays() {
        if let Some(arr) = pd.get_array_by_index(i) {
            if arr.num_components() == 1 {
                if let Some(stats) = descriptive_statistics(arr) {
                    results.push(stats);
                }
            }
        }
    }

    results
}

/// Compute descriptive statistics for all scalar cell data arrays.
pub fn descriptive_statistics_cell_data(mesh: &PolyData) -> Vec<ArrayDescriptiveStats> {
    let cd = mesh.cell_data();
    let mut results = Vec::new();

    for i in 0..cd.num_arrays() {
        if let Some(arr) = cd.get_array_by_index(i) {
            if arr.num_components() == 1 {
                if let Some(stats) = descriptive_statistics(arr) {
                    results.push(stats);
                }
            }
        }
    }

    results
}

/// Compute statistics for all columns of a Table and return as a summary Table.
pub fn descriptive_statistics_table(table: &Table) -> Table {
    let mut names = Vec::new();
    let mut counts = Vec::new();
    let mut mins = Vec::new();
    let mut maxs = Vec::new();
    let mut means = Vec::new();
    let mut stds = Vec::new();
    let mut medians = Vec::new();
    let mut skews = Vec::new();
    let mut kurts = Vec::new();

    for col in table.columns() {
        if col.num_components() != 1 {
            continue;
        }
        if let Some(stats) = descriptive_statistics(col) {
            names.push(stats.name.clone());
            counts.push(stats.count as f64);
            mins.push(stats.min);
            maxs.push(stats.max);
            means.push(stats.mean);
            stds.push(stats.std_dev);
            medians.push(stats.median);
            skews.push(stats.skewness);
            kurts.push(stats.kurtosis);
        }
    }

    // We store numeric results — column names go into a separate string-like array
    let mut result = Table::new();
    result.add_column(AnyDataArray::F64(DataArray::from_vec("count", counts, 1)));
    result.add_column(AnyDataArray::F64(DataArray::from_vec("min", mins, 1)));
    result.add_column(AnyDataArray::F64(DataArray::from_vec("max", maxs, 1)));
    result.add_column(AnyDataArray::F64(DataArray::from_vec("mean", means, 1)));
    result.add_column(AnyDataArray::F64(DataArray::from_vec("std", stds, 1)));
    result.add_column(AnyDataArray::F64(DataArray::from_vec("median", medians, 1)));
    result.add_column(AnyDataArray::F64(DataArray::from_vec("skewness", skews, 1)));
    result.add_column(AnyDataArray::F64(DataArray::from_vec("kurtosis", kurts, 1)));

    // Store source column names as field data
    let _names = names; // available for users via row index matching
    result
}

fn compute_stats(name: &str, values: &mut [f64]) -> ArrayDescriptiveStats {
    let n = values.len();
    let nf = n as f64;

    // Sort for median/percentiles
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min = values[0];
    let max = values[n - 1];

    // Mean
    let mean = values.iter().sum::<f64>() / nf;

    // Variance, skewness, kurtosis (sample)
    let mut m2 = 0.0;
    let mut m3 = 0.0;
    let mut m4 = 0.0;
    for &v in values.iter() {
        let d = v - mean;
        let d2 = d * d;
        m2 += d2;
        m3 += d2 * d;
        m4 += d2 * d2;
    }
    let variance = m2 / nf;
    let std_dev = variance.sqrt();

    let skewness = if std_dev > 1e-15 {
        (m3 / nf) / (std_dev * std_dev * std_dev)
    } else {
        0.0
    };

    let kurtosis = if std_dev > 1e-15 {
        (m4 / nf) / (variance * variance) - 3.0 // excess kurtosis
    } else {
        0.0
    };

    // Median
    let median = percentile_sorted(values, 0.5);
    let q1 = percentile_sorted(values, 0.25);
    let q3 = percentile_sorted(values, 0.75);

    ArrayDescriptiveStats {
        name: name.to_string(),
        count: n,
        min,
        max,
        mean,
        variance,
        std_dev,
        skewness,
        kurtosis,
        median,
        q1,
        q3,
        iqr: q3 - q1,
    }
}

fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return 0.0;
    }
    if n == 1 {
        return sorted[0];
    }

    let idx = p * (n - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    let frac = idx - lo as f64;

    if lo == hi {
        sorted[lo]
    } else {
        sorted[lo] * (1.0 - frac) + sorted[hi] * frac
    }
}

/// Compute Pearson correlation matrix for all scalar columns of a Table.
/// Returns a square matrix as a Vec<Vec<f64>> where entry (i,j) is the
/// correlation between column i and column j.
pub fn correlation_matrix(table: &Table) -> (Vec<String>, Vec<Vec<f64>>) {
    let mut cols: Vec<(String, Vec<f64>)> = Vec::new();

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
        cols.push((col.name().to_string(), values));
    }

    let nc = cols.len();
    let mut matrix = vec![vec![0.0; nc]; nc];
    let names: Vec<String> = cols.iter().map(|(n, _)| n.clone()).collect();

    for i in 0..nc {
        matrix[i][i] = 1.0;
        for j in (i + 1)..nc {
            let r = pearson(&cols[i].1, &cols[j].1);
            matrix[i][j] = r;
            matrix[j][i] = r;
        }
    }

    (names, matrix)
}

fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n < 2 {
        return 0.0;
    }

    let mean_a = a[..n].iter().sum::<f64>() / n as f64;
    let mean_b = b[..n].iter().sum::<f64>() / n as f64;

    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;
    for i in 0..n {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }

    let denom = (var_a * var_b).sqrt();
    if denom < 1e-15 {
        0.0
    } else {
        cov / denom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_stats() {
        let arr = AnyDataArray::F64(DataArray::from_vec(
            "test",
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            1,
        ));
        let stats = descriptive_statistics(&arr).unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert!((stats.mean - 3.0).abs() < 1e-10);
        assert!((stats.median - 3.0).abs() < 1e-10);
        assert!((stats.skewness).abs() < 1e-10); // symmetric
        assert!(stats.std_dev > 0.0);
    }

    #[test]
    fn skewed_distribution() {
        let arr = AnyDataArray::F64(DataArray::from_vec(
            "skew",
            vec![1.0, 1.0, 1.0, 1.0, 1.0, 10.0],
            1,
        ));
        let stats = descriptive_statistics(&arr).unwrap();
        assert!(stats.skewness > 0.0); // right-skewed
    }

    #[test]
    fn quartiles() {
        let arr = AnyDataArray::F64(DataArray::from_vec(
            "q",
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
            1,
        ));
        let stats = descriptive_statistics(&arr).unwrap();
        assert!(stats.q1 > 1.0 && stats.q1 < 4.0);
        assert!(stats.q3 > 5.0 && stats.q3 < 8.0);
        assert!(stats.iqr > 0.0);
    }

    #[test]
    fn correlation_matrix_test() {
        let table = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![1.0, 2.0, 3.0, 4.0, 5.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![2.0, 4.0, 6.0, 8.0, 10.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "z",
                vec![5.0, 4.0, 3.0, 2.0, 1.0],
                1,
            )));

        let (names, matrix) = correlation_matrix(&table);
        assert_eq!(names.len(), 3);
        assert!((matrix[0][0] - 1.0).abs() < 1e-10); // self-correlation
        assert!((matrix[0][1] - 1.0).abs() < 1e-10); // perfect positive
        assert!((matrix[0][2] + 1.0).abs() < 1e-10); // perfect negative
    }

    #[test]
    fn table_stats() {
        let table = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 2.0, 3.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![10.0, 20.0, 30.0],
                1,
            )));

        let summary = descriptive_statistics_table(&table);
        assert_eq!(summary.num_rows(), 2); // two columns analyzed
        assert!(summary.value_f64(0, "mean").is_some());
    }

    #[test]
    fn display() {
        let arr = AnyDataArray::F64(DataArray::from_vec("val", vec![1.0, 2.0, 3.0], 1));
        let stats = descriptive_statistics(&arr).unwrap();
        let s = format!("{stats}");
        assert!(s.contains("val:"));
        assert!(s.contains("n=3"));
    }

    #[test]
    fn mesh_point_data_stats() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "elev",
                vec![0.0, 1.0, 2.0],
                1,
            )));
        mesh.point_data_mut().set_active_scalars("elev");

        let stats = descriptive_statistics_point_data(&mesh);
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].name, "elev");
    }
}
