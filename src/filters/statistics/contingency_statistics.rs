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
    /// Joint probability matrix P(x,y).
    pub p: Vec<Vec<f64>>,
    /// Conditional probability matrix P(y|x).
    pub p_y_given_x: Vec<Vec<f64>>,
    /// Conditional probability matrix P(x|y).
    pub p_x_given_y: Vec<Vec<f64>>,
    /// Pointwise mutual information matrix.
    pub pmi: Vec<Vec<f64>>,
    /// Joint entropy H(X,Y).
    pub h_xy: f64,
    /// Conditional entropy H(Y|X).
    pub h_y_given_x: f64,
    /// Conditional entropy H(X|Y).
    pub h_x_given_y: f64,
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

    pub fn probability(&self, row: usize, col: usize) -> Option<f64> {
        self.p.get(row).and_then(|r| r.get(col)).copied()
    }

    pub fn y_given_x(&self, row: usize, col: usize) -> Option<f64> {
        self.p_y_given_x.get(row).and_then(|r| r.get(col)).copied()
    }

    pub fn x_given_y(&self, row: usize, col: usize) -> Option<f64> {
        self.p_x_given_y.get(row).and_then(|r| r.get(col)).copied()
    }

    pub fn pointwise_mutual_information(&self, row: usize, col: usize) -> Option<f64> {
        self.pmi.get(row).and_then(|r| r.get(col)).copied()
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
/// Values are binned by exact equality after conversion to `f64`.
pub fn contingency_table(table: &Table, col_a: &str, col_b: &str) -> Option<ContingencyTable> {
    let a = table.column_by_name(col_a)?;
    let b = table.column_by_name(col_b)?;
    if a.num_tuples() != b.num_tuples() {
        return None;
    }
    let n = a.num_tuples();
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
        if !vals_a.contains(&buf_a[0]) {
            vals_a.push(buf_a[0]);
        }
        if !vals_b.contains(&buf_b[0]) {
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
        let ri = vals_a.iter().position(|v| *v == buf_a[0]).unwrap();
        let ci = vals_b.iter().position(|v| *v == buf_b[0]).unwrap();
        counts[ri][ci] += 1;
    }

    let row_totals: Vec<usize> = counts.iter().map(|row| row.iter().sum()).collect();
    let col_totals: Vec<usize> = (0..nc)
        .map(|j| counts.iter().map(|row| row[j]).sum())
        .collect();
    let inv_n = 1.0 / n as f64;
    let mut p = vec![vec![0.0; nc]; nr];
    let mut p_y_given_x = vec![vec![0.0; nc]; nr];
    let mut p_x_given_y = vec![vec![0.0; nc]; nr];
    let mut pmi = vec![vec![0.0; nc]; nr];
    let mut h_xy = 0.0;
    let mut h_y_given_x = 0.0;
    let mut h_x_given_y = 0.0;

    for i in 0..nr {
        for (j, col_total) in col_totals.iter().enumerate() {
            if counts[i][j] == 0 {
                continue;
            }
            p[i][j] = inv_n * counts[i][j] as f64;
            let px = row_totals[i] as f64 * inv_n;
            let py = *col_total as f64 * inv_n;
            p_y_given_x[i][j] = p[i][j] / px;
            p_x_given_y[i][j] = p[i][j] / py;
            pmi[i][j] = (p[i][j] / (px * py)).ln();
            h_xy -= p[i][j] * p[i][j].ln();
            h_y_given_x -= p[i][j] * p_y_given_x[i][j].ln();
            h_x_given_y -= p[i][j] * p_x_given_y[i][j].ln();
        }
    }

    Some(ContingencyTable {
        row_labels: vals_a,
        col_labels: vals_b,
        counts,
        total: n,
        p,
        p_y_given_x,
        p_x_given_y,
        pmi,
        h_xy,
        h_y_given_x,
        h_x_given_y,
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
        assert!((ct.probability(0, 0).unwrap() - 2.0 / 6.0).abs() < 1e-12);
        assert!((ct.y_given_x(0, 0).unwrap() - 2.0 / 3.0).abs() < 1e-12);
        assert!((ct.x_given_y(0, 0).unwrap() - 2.0 / 3.0).abs() < 1e-12);
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
