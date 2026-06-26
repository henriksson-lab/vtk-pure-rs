//! TableToStructuredGrid — convert Table columns to a StructuredGrid.

use crate::data::{Points, StructuredGrid, Table};

/// Convert a Table with coordinate columns to a StructuredGrid.
///
/// The columns named by `x_col`, `y_col`, `z_col` supply point coordinates.
/// `dimensions` specifies the structured grid dimensions `[nx, ny, nz]`.
/// The number of rows must equal `nx * ny * nz`. Non-coordinate columns are
/// preserved as point data.
pub fn table_to_structured_grid(
    input: &Table,
    x_col: &str,
    y_col: &str,
    z_col: &str,
    dimensions: [usize; 3],
) -> Option<StructuredGrid> {
    let x_arr = input.column_by_name(x_col)?;
    let y_arr = input.column_by_name(y_col)?;
    let z_arr = input.column_by_name(z_col)?;

    let expected = dimensions[0] * dimensions[1] * dimensions[2];
    if input.num_rows() != expected
        || x_arr.num_tuples() != expected
        || y_arr.num_tuples() != expected
        || z_arr.num_tuples() != expected
    {
        return None;
    }

    let mut points = Points::<f64>::new();
    let mut bx = [0.0f64];
    let mut by = [0.0f64];
    let mut bz = [0.0f64];

    for i in 0..expected {
        x_arr.tuple_as_f64(i, &mut bx);
        y_arr.tuple_as_f64(i, &mut by);
        z_arr.tuple_as_f64(i, &mut bz);
        points.push([bx[0], by[0], bz[0]]);
    }

    let mut output = StructuredGrid::from_dimensions_and_points(dimensions, points);
    for col in input.columns() {
        if col.name() != x_col && col.name() != y_col && col.name() != z_col {
            output.point_data_mut().add_array(col.clone());
        }
    }

    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn basic_table_to_grid() {
        let mut table = Table::new();
        // 2x2x1 grid
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "X",
            vec![0.0, 1.0, 0.0, 1.0],
            1,
        )));
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "Y",
            vec![0.0, 0.0, 1.0, 1.0],
            1,
        )));
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "Z",
            vec![0.0, 0.0, 0.0, 0.0],
            1,
        )));

        let sg = table_to_structured_grid(&table, "X", "Y", "Z", [2, 2, 1]).unwrap();
        assert_eq!(sg.dimensions(), [2, 2, 1]);
        assert_eq!(sg.points.len(), 4);
        let p = sg.points.get(1);
        assert_eq!(p[0], 1.0);
    }

    #[test]
    fn insufficient_rows_returns_none() {
        let mut table = Table::new();
        table.add_column(AnyDataArray::F64(DataArray::from_vec("X", vec![0.0], 1)));
        table.add_column(AnyDataArray::F64(DataArray::from_vec("Y", vec![0.0], 1)));
        table.add_column(AnyDataArray::F64(DataArray::from_vec("Z", vec![0.0], 1)));

        let result = table_to_structured_grid(&table, "X", "Y", "Z", [2, 2, 1]);
        assert!(result.is_none());
    }

    #[test]
    fn extra_rows_returns_none() {
        let mut table = Table::new();
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "X",
            vec![0.0, 1.0, 0.0, 1.0, 2.0],
            1,
        )));
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "Y",
            vec![0.0, 0.0, 1.0, 1.0, 2.0],
            1,
        )));
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "Z",
            vec![0.0, 0.0, 0.0, 0.0, 0.0],
            1,
        )));

        let result = table_to_structured_grid(&table, "X", "Y", "Z", [2, 2, 1]);
        assert!(result.is_none());
    }

    #[test]
    fn preserves_non_coordinate_columns_as_point_data() {
        let mut table = Table::new();
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "X",
            vec![0.0, 1.0, 0.0, 1.0],
            1,
        )));
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "Y",
            vec![0.0, 0.0, 1.0, 1.0],
            1,
        )));
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "Z",
            vec![0.0, 0.0, 0.0, 0.0],
            1,
        )));
        table.add_column(AnyDataArray::F64(DataArray::from_vec(
            "value",
            vec![10.0, 20.0, 30.0, 40.0],
            1,
        )));

        let sg = table_to_structured_grid(&table, "X", "Y", "Z", [2, 2, 1]).unwrap();
        assert!(sg.point_data().get_array("value").is_some());
        assert!(sg.point_data().get_array("X").is_none());
        assert!(sg.point_data().get_array("Y").is_none());
        assert!(sg.point_data().get_array("Z").is_none());
    }
}
