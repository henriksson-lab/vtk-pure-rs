use crate::data::{AnyDataArray, DataArray, PolyData, Table};

/// Convert PolyData point coordinates and point data arrays to a Table.
///
/// Creates columns "X", "Y", "Z" from point coordinates, plus
/// one column per point data scalar array.
pub fn poly_data_to_table(input: &PolyData) -> Table {
    let n = input.points.len();
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    let mut zs = Vec::with_capacity(n);

    for i in 0..n {
        let p = input.points.get(i);
        xs.push(p[0]);
        ys.push(p[1]);
        zs.push(p[2]);
    }

    let mut table = Table::new();
    table.add_column(AnyDataArray::F64(DataArray::from_vec("X", xs, 1)));
    table.add_column(AnyDataArray::F64(DataArray::from_vec("Y", ys, 1)));
    table.add_column(AnyDataArray::F64(DataArray::from_vec("Z", zs, 1)));

    // Add scalar (1-component) point data arrays
    for i in 0..input.point_data().num_arrays() {
        let arr = input.point_data().get_array_by_index(i).unwrap();
        if arr.num_components() == 1 && arr.num_tuples() == n {
            let mut values = Vec::with_capacity(n);
            let mut buf = [0.0f64];
            for j in 0..n {
                arr.tuple_as_f64(j, &mut buf);
                values.push(buf[0]);
            }
            table.add_column(AnyDataArray::F64(DataArray::from_vec(
                arr.name(),
                values,
                1,
            )));
        }
    }

    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_conversion() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        pd.points.push([4.0, 5.0, 6.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![100.0, 200.0],
                1,
            )));

        let table = poly_data_to_table(&pd);
        assert!(table.column_by_name("X").is_some());
        assert!(table.column_by_name("Y").is_some());
        assert!(table.column_by_name("Z").is_some());
        assert!(table.column_by_name("temp").is_some());
        assert_eq!(table.num_rows(), 2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let table = poly_data_to_table(&pd);
        assert_eq!(table.num_rows(), 0);
    }

    #[test]
    fn skips_vector_arrays() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "normals",
                vec![0.0, 0.0, 1.0],
                3,
            )));
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "scalar",
                vec![42.0],
                1,
            )));

        let table = poly_data_to_table(&pd);
        assert!(table.column_by_name("scalar").is_some());
        assert!(table.column_by_name("normals").is_none()); // 3-component skipped
    }
}
