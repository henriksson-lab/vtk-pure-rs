use crate::data::traits::DataObject;
use crate::data::{AnyDataArray, FieldData};

/// Columnar data table (rows × named columns).
///
/// Analogous to VTK's `vtkTable`. Each column is an `AnyDataArray` where
/// the number of tuples is the number of rows.
///
/// # Examples
///
/// ```
/// use crate::data::{Table, AnyDataArray, DataArray};
///
/// let table = Table::new()
///     .with_column(AnyDataArray::F64(DataArray::from_vec("x", vec![1.0, 2.0, 3.0], 1)))
///     .with_column(AnyDataArray::F64(DataArray::from_vec("y", vec![4.0, 5.0, 6.0], 1)));
/// assert_eq!(table.num_rows(), 3);
/// assert_eq!(table.num_columns(), 2);
/// assert_eq!(table.value_f64(1, "x"), Some(2.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct Table {
    columns: Vec<AnyDataArray>,
    field_data: FieldData,
}

impl Table {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a column. All columns must have the same number of tuples (rows).
    pub fn add_column(&mut self, column: AnyDataArray) {
        if !self.columns.is_empty() && column.num_tuples() != self.num_rows() {
            return;
        }
        self.columns.push(column);
    }

    /// Number of rows.
    pub fn num_rows(&self) -> usize {
        self.columns.first().map(|c| c.num_tuples()).unwrap_or(0)
    }

    /// Number of columns.
    pub fn num_columns(&self) -> usize {
        self.columns.len()
    }

    /// Get column by index.
    pub fn column(&self, idx: usize) -> Option<&AnyDataArray> {
        self.columns.get(idx)
    }

    /// Get column by name.
    pub fn column_by_name(&self, name: &str) -> Option<&AnyDataArray> {
        self.columns.iter().find(|c| c.name() == name)
    }

    /// Get column names.
    pub fn column_names(&self) -> Vec<&str> {
        self.columns.iter().map(|c| c.name()).collect()
    }

    /// Iterate over columns.
    pub fn columns(&self) -> &[AnyDataArray] {
        &self.columns
    }

    /// Get a single scalar value at (row, column_name).
    pub fn value_f64(&self, row: usize, column_name: &str) -> Option<f64> {
        let col = self.column_by_name(column_name)?;
        if row >= col.num_tuples() {
            return None;
        }
        let mut buf = [0.0f64];
        col.tuple_as_f64(row, &mut buf);
        Some(buf[0])
    }

    /// Get row indices where a scalar column satisfies a predicate.
    pub fn filter_rows(&self, column_name: &str, predicate: impl Fn(f64) -> bool) -> Vec<usize> {
        let Some(col) = self.column_by_name(column_name) else {
            return Vec::new();
        };
        let mut result = Vec::new();
        let mut buf = [0.0f64];
        for i in 0..col.num_tuples() {
            col.tuple_as_f64(i, &mut buf);
            if predicate(buf[0]) {
                result.push(i);
            }
        }
        result
    }

    /// Get row indices sorted by a column's values (ascending).
    pub fn sort_by_column(&self, column_name: &str) -> Vec<usize> {
        let Some(col) = self.column_by_name(column_name) else {
            return Vec::new();
        };
        let mut indices: Vec<usize> = (0..col.num_tuples()).collect();
        let mut buf = [0.0f64];
        let mut values: Vec<f64> = Vec::with_capacity(col.num_tuples());
        for i in 0..col.num_tuples() {
            col.tuple_as_f64(i, &mut buf);
            values.push(buf[0]);
        }
        indices.sort_by(|&a, &b| {
            values[a]
                .partial_cmp(&values[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        indices
    }

    /// Extract a subset of rows by indices, returning a new Table.
    pub fn select_rows(&self, indices: &[usize]) -> Table {
        let mut result = Table::new();
        for col in &self.columns {
            let nc = col.num_components();
            let mut data = Vec::with_capacity(indices.len() * nc);
            let mut buf = vec![0.0f64; nc];
            for &idx in indices {
                if idx >= col.num_tuples() {
                    continue;
                }
                col.tuple_as_f64(idx, &mut buf);
                data.extend_from_slice(&buf);
            }
            result
                .columns
                .push(AnyDataArray::F64(crate::data::DataArray::from_vec(
                    col.name(),
                    data,
                    nc,
                )));
        }
        result
    }

    /// Remove a column by name. Returns the removed column if found.
    pub fn remove_column(&mut self, name: &str) -> Option<AnyDataArray> {
        if let Some(idx) = self.columns.iter().position(|c| c.name() == name) {
            Some(self.columns.remove(idx))
        } else {
            None
        }
    }

    /// Builder: add a column.
    pub fn with_column(mut self, column: AnyDataArray) -> Self {
        self.add_column(column);
        self
    }

    /// Write the table as CSV to a writer.
    ///
    /// ```
    /// use crate::data::{Table, AnyDataArray, DataArray};
    ///
    /// let table = Table::new()
    ///     .with_column(AnyDataArray::F64(DataArray::from_vec("x", vec![1.0, 2.0], 1)))
    ///     .with_column(AnyDataArray::F64(DataArray::from_vec("y", vec![3.0, 4.0], 1)));
    /// let mut buf = Vec::new();
    /// table.to_csv(&mut buf).unwrap();
    /// let csv = String::from_utf8(buf).unwrap();
    /// assert!(csv.contains("x,y"));
    /// ```
    pub fn to_csv<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        // Header
        let names: Vec<&str> = self.columns.iter().map(|c| c.name()).collect();
        writeln!(w, "{}", names.join(","))?;

        // Rows
        for row in 0..self.num_rows() {
            let mut vals = Vec::with_capacity(self.columns.len());
            for col in &self.columns {
                let nc = col.num_components();
                let mut buf = vec![0.0f64; nc];
                col.tuple_as_f64(row, &mut buf);
                if nc == 1 {
                    vals.push(format!("{}", buf[0]));
                } else {
                    let components: Vec<String> = buf.iter().map(|v| format!("{v}")).collect();
                    vals.push(components.join(";"));
                }
            }
            writeln!(w, "{}", vals.join(","))?;
        }
        Ok(())
    }

    /// Read a table from CSV. First line is header (column names).
    /// All values are parsed as f64.
    pub fn from_csv<R: std::io::BufRead>(r: R) -> Result<Self, String> {
        let mut lines = r.lines();

        // Header
        let header = lines
            .next()
            .ok_or("empty CSV")?
            .map_err(|e| e.to_string())?;
        let col_names: Vec<&str> = header.trim().split(',').collect();
        let ncols = col_names.len();
        let mut columns: Vec<Vec<f64>> = vec![Vec::new(); ncols];

        // Data rows
        for line_result in lines {
            let line = line_result.map_err(|e| e.to_string())?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let values: Vec<&str> = trimmed.split(',').collect();
            for (i, val) in values.iter().enumerate().take(ncols) {
                let v: f64 = val.trim().parse().unwrap_or(f64::NAN);
                columns[i].push(v);
            }
        }

        let mut table = Table::new();
        for (i, name) in col_names.iter().enumerate() {
            let arr = crate::data::DataArray::from_vec(name.trim(), columns[i].clone(), 1);
            table.add_column(AnyDataArray::F64(arr));
        }
        Ok(table)
    }

    /// Write CSV to a file path.
    pub fn write_csv_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        self.to_csv(&mut w)
    }

    /// Read CSV from a file path.
    pub fn read_csv_file(path: &std::path::Path) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let reader = std::io::BufReader::new(file);
        Self::from_csv(reader)
    }
}

impl DataObject for Table {
    fn field_data(&self) -> &FieldData {
        &self.field_data
    }

    fn field_data_mut(&mut self) -> &mut FieldData {
        &mut self.field_data
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Table: {} rows, {} columns [{}]",
            self.num_rows(),
            self.num_columns(),
            self.column_names().join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray;

    #[test]
    fn basic_table() {
        let mut table = Table::new();
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0, 2.0, 3.0],
            1,
        )));
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "y",
            vec![4.0, 5.0, 6.0],
            1,
        )));

        assert_eq!(table.num_rows(), 3);
        assert_eq!(table.num_columns(), 2);
        assert_eq!(table.column_names(), vec!["x", "y"]);
    }

    #[test]
    fn column_lookup() {
        let mut table = Table::new();
        table.add_column(AnyDataArray::I32(DataArray::from_vec(
            "ids",
            vec![10, 20, 30],
            1,
        )));
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "values",
            vec![1.1, 2.2, 3.3],
            1,
        )));

        let col = table.column_by_name("values").unwrap();
        assert_eq!(col.num_tuples(), 3);
        let mut buf = [0.0f64];
        col.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 2.2).abs() < 1e-10);
    }

    #[test]
    fn value_f64_access() {
        let table = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![10.0, 20.0, 30.0],
            1,
        )));
        assert_eq!(table.value_f64(1, "x"), Some(20.0));
        assert_eq!(table.value_f64(5, "x"), None);
        assert_eq!(table.value_f64(0, "missing"), None);
    }

    #[test]
    fn filter_rows() {
        let table = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "val",
            vec![1.0, 5.0, 2.0, 8.0, 3.0],
            1,
        )));
        let big = table.filter_rows("val", |v| v > 3.0);
        assert_eq!(big, vec![1, 3]);
    }

    #[test]
    fn sort_by_column() {
        let table = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "val",
            vec![3.0, 1.0, 4.0, 1.0, 5.0],
            1,
        )));
        let sorted = table.sort_by_column("val");
        assert_eq!(sorted[0], 1); // val=1.0
        assert_eq!(sorted[1], 3); // val=1.0
    }

    #[test]
    fn select_rows() {
        let mut table = Table::new();
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![10.0, 20.0, 30.0, 40.0],
            1,
        )));
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "y",
            vec![1.0, 2.0, 3.0, 4.0],
            1,
        )));
        let sub = table.select_rows(&[0, 2]);
        assert_eq!(sub.num_rows(), 2);
        assert_eq!(sub.value_f64(0, "x"), Some(10.0));
        assert_eq!(sub.value_f64(1, "x"), Some(30.0));
    }

    #[test]
    fn select_rows_ignores_out_of_range_indices() {
        let table = Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![10.0, 20.0],
            1,
        )));
        let sub = table.select_rows(&[1, 4]);
        assert_eq!(sub.num_rows(), 1);
        assert_eq!(sub.value_f64(0, "x"), Some(20.0));
    }

    #[test]
    fn remove_column() {
        let mut table = Table::new();
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "a",
            vec![1.0, 2.0],
            1,
        )));
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "b",
            vec![3.0, 4.0],
            1,
        )));
        assert_eq!(table.num_columns(), 2);
        table.remove_column("a");
        assert_eq!(table.num_columns(), 1);
        assert!(table.column_by_name("b").is_some());
    }

    #[test]
    fn add_column_rejects_mismatched_rows() {
        let mut table = Table::new();
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "a",
            vec![1.0, 2.0],
            1,
        )));
        table.add_column(AnyDataArray::F64(DataArray::from_vec("b", vec![3.0], 1)));
        assert_eq!(table.num_columns(), 1);
        assert!(table.column_by_name("b").is_none());
    }

    #[test]
    fn multicomponent_columns() {
        let mut table = Table::new();
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "position",
            vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0],
            3,
        )));
        assert_eq!(table.num_rows(), 2);
        assert_eq!(table.column(0).unwrap().num_components(), 3);
    }

    #[test]
    fn csv_roundtrip() {
        let table = Table::new()
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![1.0, 2.0, 3.0],
                1,
            )))
            .with_column(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![4.0, 5.0, 6.0],
                1,
            )));

        let mut buf = Vec::new();
        table.to_csv(&mut buf).unwrap();
        let csv = String::from_utf8(buf.clone()).unwrap();
        assert!(csv.starts_with("x,y\n"));

        let loaded = Table::from_csv(std::io::BufReader::new(&buf[..])).unwrap();
        assert_eq!(loaded.num_rows(), 3);
        assert_eq!(loaded.num_columns(), 2);
        assert_eq!(loaded.value_f64(0, "x"), Some(1.0));
        assert_eq!(loaded.value_f64(2, "y"), Some(6.0));
    }
}
