//! Contingency table computation for categorical data analysis.

use crate::data::{AnyDataArray, DataArray, Table};

/// A contingency table (cross-tabulation) of two categorical variables.
#[derive(Debug, Clone)]
pub struct ContingencyTable {
    /// Unique values in column A (row labels).
    pub row_labels: Vec<f64>,
    /// Unique values in column B (column labels).
    pub col_labels: Vec<f64>,
    /// Count matrix: counts[i][j] = count of (row_labels[i], col_labels[j]).
    pub counts: Vec<Vec<usize>>,
    /// Total count.
    pub total: usize,
}

impl ContingencyTable {
    /// Chi-squared statistic for independence test.
    pub fn chi_squared(&self) -> f64 {
        let row_totals: Vec<usize> = self.counts.iter().map(|row| row.iter().sum()).collect();
        let col_totals: Vec<usize> = (0..self.col_labels.len())
            .map(|j| self.counts.iter().map(|row| row[j]).sum())
            .collect();

        let mut chi2 = 0.0;
        for i in 0..self.row_labels.len() {
            for j in 0..self.col_labels.len() {
                let expected = row_totals[i] as f64 * col_totals[j] as f64 / self.total as f64;
                if expected > 0.0 {
                    let diff = self.counts[i][j] as f64 - expected;
                    chi2 += diff * diff / expected;
                }
            }
        }
        chi2
    }

    /// Cramér's V measure of association (0 = independent, 1 = perfect).
    pub fn cramers_v(&self) -> f64 {
        let chi2 = self.chi_squared();
        let k = self.row_labels.len().min(self.col_labels.len());
        if k <= 1 || self.total == 0 {
            return 0.0;
        }
        (chi2 / (self.total as f64 * (k - 1) as f64)).sqrt()
    }
}

impl std::fmt::Display for ContingencyTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ContingencyTable: {}x{}, n={}, χ²={:.4}, V={:.4}",
            self.row_labels.len(),
            self.col_labels.len(),
            self.total,
            self.chi_squared(),
            self.cramers_v()
        )
    }
}

/// Compute a contingency table from two columns of a Table.
///
/// Values are binned by equality (within tolerance for floats).
pub fn contingency_table(table: &Table, col_a: &str, col_b: &str) -> Option<ContingencyTable> {
    let a = table.column_by_name(col_a)?;
    let b = table.column_by_name(col_b)?;
    let n = a.num_tuples().min(b.num_tuples());
    if n == 0 {
        return None;
    }

    let mut buf_a = [0.0f64];
    let mut buf_b = [0.0f64];

    // Collect unique values
    let mut vals_a: Vec<f64> = Vec::new();
    let mut vals_b: Vec<f64> = Vec::new();

    for i in 0..n {
        a.tuple_as_f64(i, &mut buf_a);
        b.tuple_as_f64(i, &mut buf_b);
        if !vals_a.iter().any(|v| (*v - buf_a[0]).abs() < 1e-10) {
            vals_a.push(buf_a[0]);
        }
        if !vals_b.iter().any(|v| (*v - buf_b[0]).abs() < 1e-10) {
            vals_b.push(buf_b[0]);
        }
    }

    vals_a.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    vals_b.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));

    let nr = vals_a.len();
    let nc = vals_b.len();
    let mut counts = vec![vec![0usize; nc]; nr];

    for i in 0..n {
        a.tuple_as_f64(i, &mut buf_a);
        b.tuple_as_f64(i, &mut buf_b);
        let ri = vals_a
            .iter()
            .position(|v| (*v - buf_a[0]).abs() < 1e-10)
            .unwrap();
        let ci = vals_b
            .iter()
            .position(|v| (*v - buf_b[0]).abs() < 1e-10)
            .unwrap();
        counts[ri][ci] += 1;
    }

    Some(ContingencyTable {
        row_labels: vals_a,
        col_labels: vals_b,
        counts,
        total: n,
    })
}

/// Convert a ContingencyTable to a Table for output.
pub fn contingency_to_table(ct: &ContingencyTable) -> Table {
    let mut result = Table::new();
    for (ci, &label) in ct.col_labels.iter().enumerate() {
        let col: Vec<f64> = ct.counts.iter().map(|row| row[ci] as f64).collect();
        result.add_column(AnyDataArray::F64(DataArray::from_vec(
            &format!("{label}"),
            col,
            1,
        )));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_contingency() {
        let t = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "gender",
                vec![0.0, 0.0, 1.0, 1.0, 0.0, 1.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "choice",
                vec![1.0, 2.0, 1.0, 2.0, 1.0, 2.0],
                1,
            )));

        let ct = contingency_table(&t, "gender", "choice").unwrap();
        assert_eq!(ct.row_labels.len(), 2);
        assert_eq!(ct.col_labels.len(), 2);
        assert_eq!(ct.total, 6);
    }

    #[test]
    fn chi_squared_independent() {
        // Equal distribution → chi² ≈ 0
        let t = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![0.0, 0.0, 1.0, 1.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![0.0, 1.0, 0.0, 1.0],
                1,
            )));
        let ct = contingency_table(&t, "a", "b").unwrap();
        assert!(ct.chi_squared() < 0.01);
        assert!(ct.cramers_v() < 0.01);
    }

    #[test]
    fn perfect_association() {
        let t = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![0.0, 0.0, 1.0, 1.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![0.0, 0.0, 1.0, 1.0],
                1,
            )));
        let ct = contingency_table(&t, "a", "b").unwrap();
        assert!(ct.cramers_v() > 0.9);
    }

    #[test]
    fn display() {
        let t = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![1.0, 2.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![1.0, 2.0],
                1,
            )));
        let ct = contingency_table(&t, "x", "y").unwrap();
        let s = format!("{ct}");
        assert!(s.contains("ContingencyTable"));
    }
}
