use crate::data::{CellArray, Points, PolyData, Table};

/// Convert a Table to PolyData by interpreting columns as X, Y, Z coordinates.
///
/// Creates a point for each row using the named columns. If `z_column` is None,
/// Z is set to 0.0. Each point is also added as a vertex cell.
pub fn table_to_poly_data(
    input: &Table,
    x_column: &str,
    y_column: &str,
    z_column: Option<&str>,
) -> PolyData {
    let x_arr = match input.column_by_name(x_column) {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let y_arr = match input.column_by_name(y_column) {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let z_arr = z_column.and_then(|name| input.column_by_name(name));

    let n = x_arr.num_tuples().min(y_arr.num_tuples());
    let n = if let Some(z) = &z_arr {
        n.min(z.num_tuples())
    } else {
        n
    };

    let mut points = Points::<f64>::new();
    let mut verts = CellArray::new();
    let mut bx = [0.0f64];
    let mut by = [0.0f64];
    let mut bz = [0.0f64];

    for i in 0..n {
        x_arr.tuple_as_f64(i, &mut bx);
        y_arr.tuple_as_f64(i, &mut by);
        if let Some(z) = &z_arr {
            z.tuple_as_f64(i, &mut bz);
        } else {
            bz[0] = 0.0;
        }
        let idx = points.len() as i64;
        points.push([bx[0], by[0], bz[0]]);
        verts.push_cell(&[idx]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.verts = verts;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn basic_conversion() {
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

        let pd = table_to_poly_data(&table, "x", "y", None);
        assert_eq!(pd.points.len(), 3);
        assert_eq!(pd.verts.num_cells(), 3);

        let p = pd.points.get(0);
        assert_eq!(p[0], 1.0);
        assert_eq!(p[1], 4.0);
        assert_eq!(p[2], 0.0);
    }

    #[test]
    fn with_z() {
        let mut table = Table::new();
        table.add_column(AnyDataArray::F64(DataArray::from_vec("x", vec![1.0], 1)));
        table.add_column(AnyDataArray::F64(DataArray::from_vec("y", vec![2.0], 1)));
        table.add_column(AnyDataArray::F64(DataArray::from_vec("z", vec![3.0], 1)));

        let pd = table_to_poly_data(&table, "x", "y", Some("z"));
        let p = pd.points.get(0);
        assert_eq!(p, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn missing_column() {
        let mut table = Table::new();
        table.add_column(AnyDataArray::F64(DataArray::from_vec("x", vec![1.0], 1)));

        let pd = table_to_poly_data(&table, "x", "missing", None);
        assert_eq!(pd.points.len(), 0);
    }
}
