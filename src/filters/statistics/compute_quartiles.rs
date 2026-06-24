//! Quartile computation for data arrays.

use crate::data::{AnyDataArray, DataArray, PolyData, Table};

/// Quartile results.
#[derive(Debug, Clone)]
pub struct Quartiles {
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub iqr: f64,
    pub lower_fence: f64,
    pub upper_fence: f64,
}

impl std::fmt::Display for Quartiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "min={:.4} Q1={:.4} median={:.4} Q3={:.4} max={:.4} IQR={:.4}",
            self.min, self.q1, self.median, self.q3, self.max, self.iqr
        )
    }
}

/// Compute quartiles for a data array.
pub fn compute_quartiles_array(array: &AnyDataArray) -> Option<Quartiles> {
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
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let q1 = percentile(&values, 0.25);
    let median = percentile(&values, 0.5);
    let q3 = percentile(&values, 0.75);
    let iqr = q3 - q1;

    Some(Quartiles {
        min: values[0],
        q1,
        median,
        q3,
        max: values[n - 1],
        iqr,
        lower_fence: q1 - 1.5 * iqr,
        upper_fence: q3 + 1.5 * iqr,
    })
}

/// Compute quartiles for a named column of a Table.
pub fn compute_quartiles_column(table: &Table, column_name: &str) -> Option<Quartiles> {
    let col = table.column_by_name(column_name)?;
    compute_quartiles_array(col)
}

/// Compute quartiles for all scalar point data arrays.
pub fn compute_quartiles_point_data(mesh: &PolyData) -> Vec<(String, Quartiles)> {
    let mut results = Vec::new();
    let pd = mesh.point_data();
    for i in 0..pd.num_arrays() {
        if let Some(arr) = pd.get_array_by_index(i) {
            if arr.num_components() == 1 {
                if let Some(q) = compute_quartiles_array(arr) {
                    results.push((arr.name().to_string(), q));
                }
            }
        }
    }
    results
}

/// Compute five-number summary as a Table row.
pub fn five_number_summary(table: &Table) -> Table {
    let mut names_col = Vec::new();
    let mut min_col = Vec::new();
    let mut q1_col = Vec::new();
    let mut med_col = Vec::new();
    let mut q3_col = Vec::new();
    let mut max_col = Vec::new();

    for col in table.columns() {
        if col.num_components() != 1 {
            continue;
        }
        if let Some(q) = compute_quartiles_array(col) {
            names_col.push(col.name().to_string());
            min_col.push(q.min);
            q1_col.push(q.q1);
            med_col.push(q.median);
            q3_col.push(q.q3);
            max_col.push(q.max);
        }
    }

    let mut result = Table::new();
    result.add_column(AnyDataArray::F64(DataArray::from_vec("min", min_col, 1)));
    result.add_column(AnyDataArray::F64(DataArray::from_vec("Q1", q1_col, 1)));
    result.add_column(AnyDataArray::F64(DataArray::from_vec("median", med_col, 1)));
    result.add_column(AnyDataArray::F64(DataArray::from_vec("Q3", q3_col, 1)));
    result.add_column(AnyDataArray::F64(DataArray::from_vec("max", max_col, 1)));
    result
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    let idx = p * (n - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    let frac = idx - lo as f64;
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_quartiles() {
        let arr = AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
            1,
        ));
        let q = compute_quartiles_array(&arr).unwrap();
        assert_eq!(q.min, 1.0);
        assert_eq!(q.max, 8.0);
        assert!((q.median - 4.5).abs() < 0.01);
        assert!(q.q1 < q.median);
        assert!(q.q3 > q.median);
        assert!(q.iqr > 0.0);
    }

    #[test]
    fn fences() {
        let arr = AnyDataArray::F64(DataArray::from_vec("x", vec![1.0, 2.0, 3.0, 4.0, 5.0], 1));
        let q = compute_quartiles_array(&arr).unwrap();
        assert!(q.lower_fence < q.q1);
        assert!(q.upper_fence > q.q3);
    }

    #[test]
    fn five_number() {
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
        let summary = five_number_summary(&table);
        assert_eq!(summary.num_rows(), 2);
        assert_eq!(summary.num_columns(), 5);
    }

    #[test]
    fn display() {
        let arr = AnyDataArray::F64(DataArray::from_vec("x", vec![1.0, 2.0, 3.0], 1));
        let q = compute_quartiles_array(&arr).unwrap();
        let s = format!("{q}");
        assert!(s.contains("Q1="));
        assert!(s.contains("median="));
    }
}
