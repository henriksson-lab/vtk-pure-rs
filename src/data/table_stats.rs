use crate::data::Table;

/// Descriptive statistics for a column.
#[derive(Debug, Clone)]
pub struct ColumnStats {
    pub name: String,
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub median: f64,
}

impl std::fmt::Display for ColumnStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: n={}, min={:.4}, max={:.4}, mean={:.4}, std={:.4}, median={:.4}",
            self.name, self.count, self.min, self.max, self.mean, self.std_dev, self.median
        )
    }
}

/// Compute descriptive statistics for a Table column.
pub fn describe_column(table: &Table, column_name: &str) -> Option<ColumnStats> {
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

    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let min = values[0];
    let max = values[n - 1];
    let mean = values.iter().sum::<f64>() / n as f64;
    let variance = values.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / n as f64;
    let median = if n % 2 == 0 {
        (values[n / 2 - 1] + values[n / 2]) / 2.0
    } else {
        values[n / 2]
    };

    Some(ColumnStats {
        name: column_name.to_string(),
        count: n,
        min,
        max,
        mean,
        std_dev: variance.sqrt(),
        median,
    })
}

/// Compute Pearson correlation between two columns.
pub fn correlation(table: &Table, col_a: &str, col_b: &str) -> Option<f64> {
    let a = table.column_by_name(col_a)?;
    let b = table.column_by_name(col_b)?;
    let n = a.num_tuples().min(b.num_tuples());
    if n < 2 {
        return None;
    }

    let mut buf_a = [0.0f64];
    let mut buf_b = [0.0f64];

    let mut sum_a = 0.0;
    let mut sum_b = 0.0;
    for i in 0..n {
        a.tuple_as_f64(i, &mut buf_a);
        b.tuple_as_f64(i, &mut buf_b);
        sum_a += buf_a[0];
        sum_b += buf_b[0];
    }
    let mean_a = sum_a / n as f64;
    let mean_b = sum_b / n as f64;

    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;
    for i in 0..n {
        a.tuple_as_f64(i, &mut buf_a);
        b.tuple_as_f64(i, &mut buf_b);
        let da = buf_a[0] - mean_a;
        let db = buf_b[0] - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }

    let denom = (var_a * var_b).sqrt();
    if denom < 1e-15 {
        return Some(0.0);
    }
    Some(cov / denom)
}

/// Simple linear regression: y = slope * x + intercept.
/// Returns (slope, intercept, r_squared).
pub fn linear_regression(table: &Table, x_col: &str, y_col: &str) -> Option<(f64, f64, f64)> {
    let x = table.column_by_name(x_col)?;
    let y = table.column_by_name(y_col)?;
    let n = x.num_tuples().min(y.num_tuples());
    if n < 2 {
        return None;
    }

    let mut buf_x = [0.0f64];
    let mut buf_y = [0.0f64];
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xy = 0.0;
    let mut sum_x2 = 0.0;
    let mut _sum_y2 = 0.0;

    for i in 0..n {
        x.tuple_as_f64(i, &mut buf_x);
        y.tuple_as_f64(i, &mut buf_y);
        sum_x += buf_x[0];
        sum_y += buf_y[0];
        sum_xy += buf_x[0] * buf_y[0];
        sum_x2 += buf_x[0] * buf_x[0];
        _sum_y2 += buf_y[0] * buf_y[0];
    }

    let nf = n as f64;
    let denom = nf * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-15 {
        return None;
    }

    let slope = (nf * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / nf;

    let ss_res: f64 = (0..n)
        .map(|i| {
            x.tuple_as_f64(i, &mut buf_x);
            y.tuple_as_f64(i, &mut buf_y);
            let pred = slope * buf_x[0] + intercept;
            (buf_y[0] - pred) * (buf_y[0] - pred)
        })
        .sum();
    let mean_y = sum_y / nf;
    let ss_tot: f64 = (0..n)
        .map(|i| {
            y.tuple_as_f64(i, &mut buf_y);
            (buf_y[0] - mean_y) * (buf_y[0] - mean_y)
        })
        .sum();
    let r2 = if ss_tot > 1e-15 {
        1.0 - ss_res / ss_tot
    } else {
        1.0
    };

    Some((slope, intercept, r2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    fn make_table() -> Table {
        Table::new()
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
                vec![5.0, 3.0, 1.0, 4.0, 2.0],
                1,
            )))
    }

    #[test]
    fn describe() {
        let t = make_table();
        let stats = describe_column(&t, "x").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert!((stats.mean - 3.0).abs() < 1e-10);
        assert!((stats.median - 3.0).abs() < 1e-10);
    }

    #[test]
    fn perfect_correlation() {
        let t = make_table();
        let r = correlation(&t, "x", "y").unwrap();
        assert!((r - 1.0).abs() < 1e-10); // y = 2x, perfect positive
    }

    #[test]
    fn linear_reg() {
        let t = make_table();
        let (slope, intercept, r2) = linear_regression(&t, "x", "y").unwrap();
        assert!((slope - 2.0).abs() < 1e-10);
        assert!(intercept.abs() < 1e-10);
        assert!((r2 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn describe_display() {
        let t = make_table();
        let stats = describe_column(&t, "z").unwrap();
        let s = format!("{stats}");
        assert!(s.contains("z:"));
        assert!(s.contains("n=5"));
    }
}
