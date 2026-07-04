//! Principal Component Analysis on tabular data.
//!
//! Computes eigenvalues and eigenvectors of the covariance matrix,
//! projects data onto principal components, and reports explained variance.

use crate::data::{AnyDataArray, DataArray, Table};

/// Result of a PCA computation.
#[derive(Debug, Clone)]
pub struct PcaResult {
    /// Eigenvalues sorted descending (variance along each PC).
    pub eigenvalues: Vec<f64>,
    /// Eigenvectors as rows (each row is a principal component direction).
    pub eigenvectors: Vec<Vec<f64>>,
    /// Column means used for centering.
    pub means: Vec<f64>,
    /// Explained variance ratio for each component.
    pub explained_variance_ratio: Vec<f64>,
    /// Column names from the input.
    pub column_names: Vec<String>,
}

/// Compute PCA on scalar columns of a Table.
///
/// Returns eigenvalues, eigenvectors, and explained variance ratios.
/// Uses the power iteration method for eigendecomposition.
pub fn pca(table: &Table, max_components: usize) -> Option<PcaResult> {
    // Extract scalar columns as matrix
    let mut cols: Vec<Vec<f64>> = Vec::new();
    let mut names: Vec<String> = Vec::new();

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
        names.push(col.name().to_string());
    }

    let p = cols.len(); // number of features
    if p == 0 {
        return None;
    }
    let n = cols[0].len(); // number of samples
    if n < 2 {
        return None;
    }

    // Compute means
    let means: Vec<f64> = cols
        .iter()
        .map(|c| c.iter().sum::<f64>() / n as f64)
        .collect();

    // Compute covariance matrix (p x p)
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

    let num_components = max_components.min(p);
    let mut components = jacobi_eigen_decomposition(cov);
    components.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let total_variance: f64 = components.iter().map(|(eval, _)| eval.max(0.0)).sum();

    let mut eigenvalues = Vec::new();
    let mut eigenvectors = Vec::new();
    for (eval, evec) in components.into_iter().take(num_components) {
        let eval = eval.max(0.0);
        eigenvalues.push(eval);
        eigenvectors.push(evec);
    }

    let explained_variance_ratio = if total_variance > 1e-15 {
        eigenvalues.iter().map(|e| e / total_variance).collect()
    } else {
        vec![0.0; eigenvalues.len()]
    };

    Some(PcaResult {
        eigenvalues,
        eigenvectors,
        means,
        explained_variance_ratio,
        column_names: names,
    })
}

/// Project table data onto the first N principal components.
///
/// Returns a new Table with columns "PC1", "PC2", etc.
pub fn pca_project(table: &Table, pca_result: &PcaResult) -> Table {
    let cols = extract_scalar_columns(table);
    let n = if cols.is_empty() { 0 } else { cols[0].len() };
    let p = cols.len();
    let nc = pca_result.eigenvectors.len();

    let mut projected: Vec<Vec<f64>> = vec![Vec::with_capacity(n); nc];

    for row in 0..n {
        for (ci, evec) in pca_result.eigenvectors.iter().enumerate() {
            let mut val = 0.0;
            for j in 0..p.min(evec.len()) {
                val += (cols[j][row] - pca_result.means[j]) * evec[j];
            }
            projected[ci].push(val);
        }
    }

    let mut result = Table::new();
    for (ci, data) in projected.into_iter().enumerate() {
        result.add_column(AnyDataArray::F64(DataArray::from_vec(
            &format!("PC{}", ci + 1),
            data,
            1,
        )));
    }
    result
}

fn extract_scalar_columns(table: &Table) -> Vec<Vec<f64>> {
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
        cols.push(values);
    }
    cols
}

fn jacobi_eigen_decomposition(mut matrix: Vec<Vec<f64>>) -> Vec<(f64, Vec<f64>)> {
    let n = matrix.len();
    if n == 0 {
        return Vec::new();
    }

    let mut eigenvectors = vec![vec![0.0; n]; n];
    for i in 0..n {
        eigenvectors[i][i] = 1.0;
    }

    for _ in 0..(100 * n * n) {
        let mut p = 0;
        let mut q = 0;
        let mut max_offdiag = 0.0f64;
        for i in 0..n {
            for j in i + 1..n {
                let value = matrix[i][j].abs();
                if value > max_offdiag {
                    max_offdiag = value;
                    p = i;
                    q = j;
                }
            }
        }
        if max_offdiag < 1e-12 {
            break;
        }

        let app = matrix[p][p];
        let aqq = matrix[q][q];
        let apq = matrix[p][q];
        let theta = 0.5 * (2.0 * apq).atan2(aqq - app);
        let c = theta.cos();
        let s = theta.sin();

        for i in 0..n {
            if i != p && i != q {
                let aip = matrix[i][p];
                let aiq = matrix[i][q];
                matrix[i][p] = c * aip - s * aiq;
                matrix[p][i] = matrix[i][p];
                matrix[i][q] = s * aip + c * aiq;
                matrix[q][i] = matrix[i][q];
            }
        }

        matrix[p][p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        matrix[q][q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        matrix[p][q] = 0.0;
        matrix[q][p] = 0.0;

        for row in &mut eigenvectors {
            let vip = row[p];
            let viq = row[q];
            row[p] = c * vip - s * viq;
            row[q] = s * vip + c * viq;
        }
    }

    (0..n)
        .map(|i| {
            let eigenvector = (0..n).map(|row| eigenvectors[row][i]).collect();
            (matrix[i][i], eigenvector)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_pca() {
        // Correlated data: y ≈ 2x
        let table = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![1.0, 2.0, 3.0, 4.0, 5.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![2.1, 3.9, 6.0, 8.1, 9.9],
                1,
            )));

        let result = pca(&table, 2).unwrap();
        assert_eq!(result.eigenvalues.len(), 2);
        assert!(result.eigenvalues[0] > result.eigenvalues[1]); // first PC explains most
        assert!(result.explained_variance_ratio[0] > 0.9); // highly correlated
    }

    #[test]
    fn pca_projection() {
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
            )));

        let result = pca(&table, 2).unwrap();
        let projected = pca_project(&table, &result);
        assert_eq!(projected.num_rows(), 5);
        assert_eq!(projected.num_columns(), 2);
        assert!(projected.column_names().contains(&"PC1"));
    }

    #[test]
    fn uncorrelated_data() {
        let table = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![1.0, 0.0, -1.0, 0.0, 2.0, -2.0, 0.5, -0.5],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 1.5, -1.5],
                1,
            )));

        let result = pca(&table, 2).unwrap();
        assert!(result.eigenvalues.len() >= 1);
        // First PC should explain less than 100% for uncorrelated data
        assert!(result.explained_variance_ratio[0] < 0.99);
    }

    #[test]
    fn single_column() {
        let table = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0, 2.0, 3.0],
            1,
        )));
        let result = pca(&table, 1).unwrap();
        assert_eq!(result.eigenvalues.len(), 1);
    }
}
