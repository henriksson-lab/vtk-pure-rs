//! Pairwise correlation matrices and covariance analysis.

use crate::data::{AnyDataArray, DataArray, Table};

/// Compute the covariance matrix for all scalar columns.
/// Returns (column_names, covariance_matrix).
pub fn covariance_matrix(table: &Table) -> (Vec<String>, Vec<Vec<f64>>) {
    let (names, cols) = extract_columns(table);
    let p = cols.len();
    if p == 0 {
        return (names, Vec::new());
    }
    let n = cols[0].len();
    if n < 2 {
        return (names, vec![vec![0.0; p]; p]);
    }

    let means: Vec<f64> = cols
        .iter()
        .map(|c| c.iter().sum::<f64>() / n as f64)
        .collect();
    let mut cov = vec![vec![0.0; p]; p];
    for i in 0..p {
        for j in i..p {
            let mut s = 0.0;
            for k in 0..n {
                s += (cols[i][k] - means[i]) * (cols[j][k] - means[j]);
            }
            cov[i][j] = s / (n - 1) as f64;
            cov[j][i] = cov[i][j];
        }
    }
    (names, cov)
}

/// Compute the Pearson correlation matrix for all scalar columns.
pub fn correlation_matrix(table: &Table) -> (Vec<String>, Vec<Vec<f64>>) {
    let (names, cov) = covariance_matrix(table);
    let p = cov.len();
    if p == 0 {
        return (names, cov);
    }

    let mut corr = vec![vec![0.0; p]; p];
    for i in 0..p {
        for j in 0..p {
            let denom = (cov[i][i] * cov[j][j]).sqrt();
            corr[i][j] = if denom > 1e-15 {
                cov[i][j] / denom
            } else {
                0.0
            };
        }
    }
    (names, corr)
}

/// Compute the Spearman rank correlation matrix.
pub fn spearman_correlation(table: &Table) -> (Vec<String>, Vec<Vec<f64>>) {
    let (names, cols) = extract_columns(table);
    let p = cols.len();
    if p == 0 {
        return (names, Vec::new());
    }
    let n = cols[0].len();
    if n < 2 {
        return (names, vec![vec![0.0; p]; p]);
    }

    // Convert each column to ranks
    let ranked: Vec<Vec<f64>> = cols.iter().map(|c| rank_values(c)).collect();

    // Compute Pearson correlation on ranks
    let means: Vec<f64> = ranked
        .iter()
        .map(|r| r.iter().sum::<f64>() / n as f64)
        .collect();
    let mut corr = vec![vec![0.0; p]; p];
    for i in 0..p {
        for j in i..p {
            let mut cov = 0.0;
            let mut var_i = 0.0;
            let mut var_j = 0.0;
            for k in 0..n {
                let di = ranked[i][k] - means[i];
                let dj = ranked[j][k] - means[j];
                cov += di * dj;
                var_i += di * di;
                var_j += dj * dj;
            }
            let denom = (var_i * var_j).sqrt();
            corr[i][j] = if denom > 1e-15 { cov / denom } else { 0.0 };
            corr[j][i] = corr[i][j];
        }
    }
    (names, corr)
}

/// Output correlation matrix as a Table.
pub fn correlation_matrix_to_table(names: &[String], matrix: &[Vec<f64>]) -> Table {
    let mut result = Table::new();
    for (ci, name) in names.iter().enumerate() {
        let col: Vec<f64> = matrix.iter().map(|row| row[ci]).collect();
        result.add_column(AnyDataArray::F64(DataArray::from_vec(name, col, 1)));
    }
    result
}

fn extract_columns(table: &Table) -> (Vec<String>, Vec<Vec<f64>>) {
    let mut names = Vec::new();
    let mut cols = Vec::new();
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
        names.push(col.name().to_string());
        cols.push(values);
    }
    (names, cols)
}

fn rank_values(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut indexed: Vec<(usize, f64)> = values.iter().enumerate().map(|(i, &v)| (i, v)).collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0f64; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j < n && (indexed[j].1 - indexed[i].1).abs() < 1e-15 {
            j += 1;
        }
        let avg = (i + j + 1) as f64 / 2.0;
        for k in i..j {
            ranks[indexed[k].0] = avg;
        }
        i = j;
    }
    ranks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_correlation() {
        let t = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![1.0, 2.0, 3.0, 4.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![2.0, 4.0, 6.0, 8.0],
                1,
            )));
        let (_, corr) = correlation_matrix(&t);
        assert!((corr[0][1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn covariance() {
        let t = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0, 2.0, 3.0],
            1,
        )));
        let (_, cov) = covariance_matrix(&t);
        assert!((cov[0][0] - 1.0).abs() < 1e-10); // var of [1,2,3] = 1
    }

    #[test]
    fn spearman() {
        let t = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![1.0, 2.0, 3.0, 4.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![10.0, 20.0, 30.0, 40.0],
                1,
            )));
        let (_, corr) = spearman_correlation(&t);
        assert!((corr[0][1] - 1.0).abs() < 1e-10); // monotonic
    }

    #[test]
    fn to_table() {
        let t = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 2.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![3.0, 4.0],
                1,
            )));
        let (names, matrix) = correlation_matrix(&t);
        let result = correlation_matrix_to_table(&names, &matrix);
        assert_eq!(result.num_rows(), 2);
        assert_eq!(result.num_columns(), 2);
    }
}
