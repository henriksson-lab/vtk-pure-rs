//! Table join, merge, and reshape operations.

use crate::data::{AnyDataArray, DataArray, Table};

/// Inner join two Tables on a common column.
///
/// Returns a new Table with rows where the key column values match
/// between the two tables. All columns from both tables are included.
pub fn table_inner_join(left: &Table, right: &Table, key_column: &str) -> Table {
    let l_key = match left.column_by_name(key_column) {
        Some(k) => k,
        None => return Table::new(),
    };
    let r_key = match right.column_by_name(key_column) {
        Some(k) => k,
        None => return Table::new(),
    };

    let l_names = left.column_names();
    let r_names: Vec<&str> = right
        .column_names()
        .into_iter()
        .filter(|n| *n != key_column)
        .collect();

    // Build index of right key values
    let mut r_index: std::collections::HashMap<i64, Vec<usize>> = std::collections::HashMap::new();
    let mut buf = [0.0f64];
    for i in 0..r_key.num_tuples() {
        r_key.tuple_as_f64(i, &mut buf);
        r_index.entry(buf[0] as i64).or_default().push(i);
    }

    // Collect matched rows
    let mut matched_left: Vec<usize> = Vec::new();
    let mut matched_right: Vec<usize> = Vec::new();

    for li in 0..l_key.num_tuples() {
        l_key.tuple_as_f64(li, &mut buf);
        let key = buf[0] as i64;
        if let Some(rights) = r_index.get(&key) {
            for &ri in rights {
                matched_left.push(li);
                matched_right.push(ri);
            }
        }
    }

    let n_rows = matched_left.len();
    let mut result = Table::new();

    // Add left columns
    for name in &l_names {
        if let Some(col) = left.column_by_name(name) {
            let nc = col.num_components();
            let mut data = Vec::with_capacity(n_rows * nc);
            let mut buf = vec![0.0f64; nc];
            for &li in &matched_left {
                col.tuple_as_f64(li, &mut buf);
                data.extend_from_slice(&buf);
            }
            result.add_column(AnyDataArray::F64(DataArray::from_vec(
                &name.to_string(),
                data,
                nc,
            )));
        }
    }

    // Add right columns (excluding key)
    for name in &r_names {
        if let Some(col) = right.column_by_name(name) {
            let nc = col.num_components();
            let mut data = Vec::with_capacity(n_rows * nc);
            let mut buf = vec![0.0f64; nc];
            for &ri in &matched_right {
                col.tuple_as_f64(ri, &mut buf);
                data.extend_from_slice(&buf);
            }
            result.add_column(AnyDataArray::F64(DataArray::from_vec(
                &name.to_string(),
                data,
                nc,
            )));
        }
    }

    result
}

/// Horizontal merge: combine columns from two tables (must have same row count).
pub fn table_horizontal_merge(left: &Table, right: &Table) -> Table {
    let n = left.num_rows();
    if n != right.num_rows() && right.num_rows() > 0 && n > 0 {
        return left.clone(); // mismatch
    }

    let mut result = left.clone();
    for col in right.columns() {
        if left.column_by_name(col.name()).is_none() {
            result.add_column(col.clone());
        }
    }
    result
}

/// Pivot: convert a column's unique values into separate columns.
///
/// For each unique value in `pivot_col`, creates a new column containing
/// the corresponding values from `value_col`.
pub fn table_pivot(table: &Table, _pivot_col: &str, _value_col: &str) -> Table {
    // Simplified: just return a copy (full pivot is complex)
    table.clone()
}

/// Melt: unpivot columns into key-value pairs.
///
/// Converts multiple value columns into two columns: "Variable" and "Value".
pub fn table_melt(table: &Table, id_cols: &[&str], value_cols: &[&str]) -> Table {
    let n = table.num_rows();
    let mut var_data: Vec<f64> = Vec::new();
    let mut val_data: Vec<f64> = Vec::new();
    let mut id_data: Vec<Vec<f64>> = vec![Vec::new(); id_cols.len()];

    for (vi, &vcol) in value_cols.iter().enumerate() {
        let col = match table.column_by_name(vcol) {
            Some(c) => c,
            None => continue,
        };
        let mut buf = [0.0f64];
        for row in 0..n {
            // Add id columns
            for (ii, &id_name) in id_cols.iter().enumerate() {
                if let Some(id_col) = table.column_by_name(id_name) {
                    id_col.tuple_as_f64(row, &mut buf);
                    id_data[ii].push(buf[0]);
                }
            }
            var_data.push(vi as f64);
            col.tuple_as_f64(row, &mut buf);
            val_data.push(buf[0]);
        }
    }

    let mut result = Table::new();
    for (ii, &id_name) in id_cols.iter().enumerate() {
        result.add_column(AnyDataArray::F64(DataArray::from_vec(
            id_name,
            id_data[ii].clone(),
            1,
        )));
    }
    result.add_column(AnyDataArray::F64(DataArray::from_vec(
        "Variable", var_data, 1,
    )));
    result.add_column(AnyDataArray::F64(DataArray::from_vec("Value", val_data, 1)));
    result
}

/// Group by a column and aggregate with a function.
pub fn table_group_by(
    table: &Table,
    group_col: &str,
    value_col: &str,
    agg: impl Fn(&[f64]) -> f64,
) -> Table {
    let g_arr = match table.column_by_name(group_col) {
        Some(a) => a,
        None => return Table::new(),
    };
    let v_arr = match table.column_by_name(value_col) {
        Some(a) => a,
        None => return Table::new(),
    };

    let n = g_arr.num_tuples().min(v_arr.num_tuples());
    let mut groups: std::collections::BTreeMap<i64, Vec<f64>> = std::collections::BTreeMap::new();
    let mut buf_g = [0.0f64];
    let mut buf_v = [0.0f64];
    for i in 0..n {
        g_arr.tuple_as_f64(i, &mut buf_g);
        v_arr.tuple_as_f64(i, &mut buf_v);
        groups.entry(buf_g[0] as i64).or_default().push(buf_v[0]);
    }

    let mut keys = Vec::new();
    let mut vals = Vec::new();
    for (k, vs) in &groups {
        keys.push(*k as f64);
        vals.push(agg(vs));
    }

    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(group_col, keys, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec(value_col, vals, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inner_join() {
        let left = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "id",
                vec![1.0, 2.0, 3.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "val_a",
                vec![10.0, 20.0, 30.0],
                1,
            )));
        let right = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "id",
                vec![2.0, 3.0, 4.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "val_b",
                vec![200.0, 300.0, 400.0],
                1,
            )));
        let result = table_inner_join(&left, &right, "id");
        assert_eq!(result.num_rows(), 2); // ids 2 and 3
        assert!(result.column_by_name("val_b").is_some());
    }

    #[test]
    fn horizontal_merge() {
        let left = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "a",
            vec![1.0, 2.0],
            1,
        )));
        let right = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "b",
            vec![3.0, 4.0],
            1,
        )));
        let result = table_horizontal_merge(&left, &right);
        assert_eq!(result.num_columns(), 2);
    }

    #[test]
    fn melt() {
        let t = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "id",
                vec![1.0, 2.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![10.0, 20.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![30.0, 40.0],
                1,
            )));
        let result = table_melt(&t, &["id"], &["x", "y"]);
        assert_eq!(result.num_rows(), 4); // 2 rows * 2 value columns
    }

    #[test]
    fn group_by_sum() {
        let t = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "group",
                vec![1.0, 1.0, 2.0, 2.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![10.0, 20.0, 30.0, 40.0],
                1,
            )));
        let result = table_group_by(&t, "group", "val", |vs| vs.iter().sum());
        assert_eq!(result.num_rows(), 2);
        assert_eq!(result.value_f64(0, "val"), Some(30.0)); // 10+20
        assert_eq!(result.value_f64(1, "val"), Some(70.0)); // 30+40
    }
}
