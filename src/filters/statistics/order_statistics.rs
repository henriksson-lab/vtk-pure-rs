//! Order statistics: quantiles, rank computation, and rank-based tests.

use crate::data::{AnyDataArray, DataArray, Table};

/// Compute arbitrary quantiles for a column.
///
/// `quantiles` should be values in [0, 1], e.g. [0.25, 0.5, 0.75] for quartiles.
/// Returns the quantile values in the same order.
pub fn compute_quantiles(table: &Table, column_name: &str, quantiles: &[f64]) -> Option<Vec<f64>> {
    let col = table.column_by_name(column_name)?;
    let n = col.num_tuples();
    if n == 0 {
        return None;
    }

    let mut values = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        col.tuple_as_f64(i, &mut buf);
        values.push(buf[0]);
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    Some(
        quantiles
            .iter()
            .map(|&q| {
                let q = q.clamp(0.0, 1.0);
                let idx = q * (n - 1) as f64;
                let lo = idx.floor() as usize;
                let hi = idx.ceil() as usize;
                let frac = idx - lo as f64;
                if lo == hi {
                    values[lo]
                } else {
                    values[lo] * (1.0 - frac) + values[hi] * frac
                }
            })
            .collect(),
    )
}

/// Compute quartiles (Q1, median, Q3) for a column.
pub fn compute_quartiles(table: &Table, column_name: &str) -> Option<(f64, f64, f64)> {
    let q = compute_quantiles(table, column_name, &[0.25, 0.5, 0.75])?;
    Some((q[0], q[1], q[2]))
}

/// Compute ranks for a column (1-based, average for ties).
pub fn compute_ranks(table: &Table, column_name: &str) -> Option<Vec<f64>> {
    let col = table.column_by_name(column_name)?;
    let n = col.num_tuples();
    if n == 0 {
        return None;
    }

    let mut indexed: Vec<(usize, f64)> = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        col.tuple_as_f64(i, &mut buf);
        indexed.push((i, buf[0]));
    }
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut ranks = vec![0.0f64; n];

    // Handle ties by averaging ranks
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j < n && (indexed[j].1 - indexed[i].1).abs() < 1e-15 {
            j += 1;
        }
        let avg_rank = (i + j + 1) as f64 / 2.0; // 1-based average
        for k in i..j {
            ranks[indexed[k].0] = avg_rank;
        }
        i = j;
    }

    Some(ranks)
}

/// Add a rank column to a Table.
pub fn add_rank_column(table: &Table, column_name: &str) -> Table {
    let ranks = match compute_ranks(table, column_name) {
        Some(r) => r,
        None => return table.clone(),
    };

    let mut result = table.clone();
    result.add_column(AnyDataArray::F64(DataArray::from_vec(
        &format!("{column_name}_rank"),
        ranks,
        1,
    )));
    result
}

/// Detect outliers using the IQR method.
///
/// Returns indices of values below Q1 - k*IQR or above Q3 + k*IQR.
/// Standard k = 1.5 for mild outliers, k = 3.0 for extreme.
pub fn iqr_outliers(table: &Table, column_name: &str, k: f64) -> Option<Vec<usize>> {
    let (q1, _, q3) = compute_quartiles(table, column_name)?;
    let iqr = q3 - q1;
    let lower = q1 - k * iqr;
    let upper = q3 + k * iqr;

    let col = table.column_by_name(column_name)?;
    let mut outliers = Vec::new();
    let mut buf = [0.0f64];
    for i in 0..col.num_tuples() {
        col.tuple_as_f64(i, &mut buf);
        if buf[0] < lower || buf[0] > upper {
            outliers.push(i);
        }
    }
    Some(outliers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_table() -> Table {
        Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
            1,
        )))
    }

    #[test]
    fn quartiles() {
        let t = make_table();
        let (q1, median, q3) = compute_quartiles(&t, "x").unwrap();
        assert!((median - 4.5).abs() < 0.1);
        assert!(q1 < median);
        assert!(q3 > median);
    }

    #[test]
    fn custom_quantiles() {
        let t = make_table();
        let q = compute_quantiles(&t, "x", &[0.0, 0.5, 1.0]).unwrap();
        assert!((q[0] - 1.0).abs() < 0.01);
        assert!((q[2] - 8.0).abs() < 0.01);
    }

    #[test]
    fn ranks() {
        let t = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![30.0, 10.0, 20.0],
            1,
        )));
        let ranks = compute_ranks(&t, "x").unwrap();
        assert!((ranks[0] - 3.0).abs() < 0.01); // 30 is largest
        assert!((ranks[1] - 1.0).abs() < 0.01); // 10 is smallest
        assert!((ranks[2] - 2.0).abs() < 0.01); // 20 is middle
    }

    #[test]
    fn tied_ranks() {
        let t = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0, 1.0, 3.0],
            1,
        )));
        let ranks = compute_ranks(&t, "x").unwrap();
        assert!((ranks[0] - 1.5).abs() < 0.01); // average of rank 1 and 2
        assert!((ranks[1] - 1.5).abs() < 0.01);
    }

    #[test]
    fn outlier_detection() {
        let t = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0],
            1,
        )));
        let outliers = iqr_outliers(&t, "x", 1.5).unwrap();
        assert!(!outliers.is_empty());
        assert!(outliers.contains(&5)); // 100.0 is an outlier
    }

    #[test]
    fn add_rank() {
        let t = make_table();
        let result = add_rank_column(&t, "x");
        assert!(result.column_by_name("x_rank").is_some());
    }
}
