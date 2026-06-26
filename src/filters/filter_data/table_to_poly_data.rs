use crate::data::{CellArray, Points, PolyData, Table};

/// Convert a Table to PolyData by interpreting columns as X, Y, Z coordinates.
///
/// Creates a point for each row using the named columns. If `z_column` is None,
/// Z is set to 0.0 and 2D points are created. A single poly-vertex cell is
/// created to reference all points, and non-coordinate columns are preserved as
/// point data.
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
    let z_arr = match z_column {
        Some(name) => match input.column_by_name(name) {
            Some(a) => Some(a),
            None => return PolyData::new(),
        },
        None => None,
    };

    let n = input.num_rows();
    if x_arr.num_tuples() != n || y_arr.num_tuples() != n {
        return PolyData::new();
    }
    if let Some(z) = &z_arr {
        if z.num_tuples() != n {
            return PolyData::new();
        }
    };

    let mut points = Points::<f64>::new();
    let mut pt_ids = Vec::with_capacity(n);
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
        pt_ids.push(idx);
    }

    let mut verts = CellArray::new();
    if !pt_ids.is_empty() {
        verts.push_cell(&pt_ids);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.verts = verts;
    for col in input.columns() {
        if col.name() != x_column && col.name() != y_column && Some(col.name()) != z_column {
            pd.point_data_mut().add_array(col.clone());
        }
    }
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
        assert_eq!(pd.verts.num_cells(), 1);

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

    #[test]
    fn missing_requested_z_column_returns_empty() {
        let mut table = Table::new();
        table.add_column(AnyDataArray::F64(DataArray::from_vec("x", vec![1.0], 1)));
        table.add_column(AnyDataArray::F64(DataArray::from_vec("y", vec![2.0], 1)));

        let pd = table_to_poly_data(&table, "x", "y", Some("missing"));
        assert_eq!(pd.points.len(), 0);
    }

    #[test]
    fn preserves_non_coordinate_columns_as_point_data() {
        let mut table = Table::new();
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "x",
            vec![1.0, 2.0],
            1,
        )));
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "y",
            vec![3.0, 4.0],
            1,
        )));
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "value",
            vec![5.0, 6.0],
            1,
        )));

        let pd = table_to_poly_data(&table, "x", "y", None);
        assert!(pd.point_data().get_array("value").is_some());
        assert!(pd.point_data().get_array("x").is_none());
        assert!(pd.point_data().get_array("y").is_none());
    }
}
