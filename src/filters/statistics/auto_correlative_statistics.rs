//! Autocorrelation analysis for time series data.

use crate::data::{AnyDataArray, DataArray, Table};

/// Compute the autocorrelation function of a column up to `max_lag` steps.
///
/// Returns a vector of autocorrelation coefficients for lags 0..=max_lag.
/// Lag 0 is always 1.0 (self-correlation).
pub fn autocorrelation(table: &Table, column_name: &str, max_lag: usize) -> Option<Vec<f64>> {
    let col = table.column_by_name(column_name)?;
    let n = col.num_tuples();
    if n < 2 {
        return None;
    }

    let mut values = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        col.tuple_as_f64(i, &mut buf);
        values.push(buf[0]);
    }

    let mean = values.iter().sum::<f64>() / n as f64;
    let variance: f64 = values.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>();

    if variance < 1e-15 {
        return Some(vec![1.0; max_lag + 1]);
    }

    let mut result = Vec::with_capacity(max_lag + 1);
    for lag in 0..=max_lag.min(n - 1) {
        let mut sum = 0.0;
        for i in 0..n - lag {
            sum += (values[i] - mean) * (values[i + lag] - mean);
        }
        result.push(sum / variance);
    }
    Some(result)
}

/// Compute autocorrelation and return as a Table with columns "Lag" and "ACF".
pub fn autocorrelation_table(table: &Table, column_name: &str, max_lag: usize) -> Table {
    let acf = match autocorrelation(table, column_name, max_lag) {
        Some(a) => a,
        None => return Table::new(),
    };

    let lags: Vec<f64> = (0..acf.len()).map(|i| i as f64).collect();
    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec("Lag", lags, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec("ACF", acf, 1)))
}

/// Compute partial autocorrelation using the Durbin-Levinson recursion.
pub fn partial_autocorrelation(
    table: &Table,
    column_name: &str,
    max_lag: usize,
) -> Option<Vec<f64>> {
    let acf = autocorrelation(table, column_name, max_lag)?;
    let n_lags = acf.len();
    if n_lags <= 1 {
        return Some(vec![1.0]);
    }

    let mut pacf = vec![0.0; n_lags];
    pacf[0] = 1.0;
    if n_lags > 1 {
        pacf[1] = acf[1];
    }

    let mut phi = vec![vec![0.0; n_lags]; n_lags];
    if n_lags > 1 {
        phi[1][1] = acf[1];
    }

    for k in 2..n_lags {
        let mut num = acf[k];
        for j in 1..k {
            num -= phi[k - 1][j] * acf[k - j];
        }
        let mut den = 1.0;
        for j in 1..k {
            den -= phi[k - 1][j] * acf[j];
        }
        if den.abs() < 1e-15 {
            break;
        }
        phi[k][k] = num / den;
        pacf[k] = phi[k][k];

        for j in 1..k {
            phi[k][j] = phi[k - 1][j] - phi[k][k] * phi[k - 1][k - j];
        }
    }
    Some(pacf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_series() {
        let table = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![5.0; 10],
            1,
        )));
        let acf = autocorrelation(&table, "x", 5).unwrap();
        assert!((acf[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn sinusoidal_series() {
        let n = 100;
        let values: Vec<f64> = (0..n).map(|i| (i as f64 * 0.1).sin()).collect();
        let table =
            Table::new().with_column(AnyDataArray::F64(DataArray::from_vec("x", values, 1)));

        let acf = autocorrelation(&table, "x", 20).unwrap();
        assert!((acf[0] - 1.0).abs() < 1e-10);
        // Sinusoid should show periodic autocorrelation
        assert!(acf.len() == 21);
    }

    #[test]
    fn acf_to_table() {
        let table = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            1,
        )));
        let result = autocorrelation_table(&table, "x", 3);
        assert_eq!(result.num_rows(), 4);
        assert!(result.column_by_name("Lag").is_some());
        assert!(result.column_by_name("ACF").is_some());
    }

    #[test]
    fn pacf() {
        let values: Vec<f64> = (0..50).map(|i| i as f64 + (i as f64 * 0.5).sin()).collect();
        let table =
            Table::new().with_column(AnyDataArray::F64(DataArray::from_vec("x", values, 1)));
        let pacf = partial_autocorrelation(&table, "x", 10).unwrap();
        assert!((pacf[0] - 1.0).abs() < 1e-10);
        assert!(pacf.len() > 1);
    }
}
