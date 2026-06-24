use crate::data::PolyData;

/// Copy a named array from field data to point data.
///
/// If the array's tuple count matches the number of points, it is
/// copied as point data. Otherwise, nothing happens.
pub fn field_to_point_data(input: &PolyData, array_name: &str) -> PolyData {
    let mut pd = input.clone();

    if let Some(arr) = input.field_data().get_array(array_name) {
        if arr.num_tuples() == input.points.len() {
            pd.point_data_mut().add_array(arr.clone());
        }
    }

    pd
}

/// Copy a named array from field data to cell data.
///
/// If the array's tuple count matches the number of cells, it is
/// copied as cell data.
pub fn field_to_cell_data(input: &PolyData, array_name: &str) -> PolyData {
    let mut pd = input.clone();

    if let Some(arr) = input.field_data().get_array(array_name) {
        if arr.num_tuples() == input.polys.num_cells() {
            pd.cell_data_mut().add_array(arr.clone());
        }
    }

    pd
}

/// Copy a named array from point data to field data.
pub fn point_data_to_field(input: &PolyData, array_name: &str) -> PolyData {
    let mut pd = input.clone();

    if let Some(arr) = input.point_data().get_array(array_name) {
        pd.field_data_mut().add_array(arr.clone());
    }

    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn field_to_point() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.field_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temps",
                vec![100.0, 200.0],
                1,
            )));

        let result = field_to_point_data(&pd, "temps");
        assert!(result.point_data().get_array("temps").is_some());
    }

    #[test]
    fn field_to_cell() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.field_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("area", vec![0.5], 1)));

        let result = field_to_cell_data(&pd, "area");
        assert!(result.cell_data().get_array("area").is_some());
    }

    #[test]
    fn point_to_field() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", vec![42.0], 1)));

        let result = point_data_to_field(&pd, "val");
        assert!(result.field_data().get_array("val").is_some());
    }

    #[test]
    fn wrong_size_skipped() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.field_data_mut().add_array(AnyDataArray::F64(
            DataArray::from_vec("wrong", vec![1.0, 2.0, 3.0], 1), // 3 tuples, 1 point
        ));

        let result = field_to_point_data(&pd, "wrong");
        assert!(result.point_data().get_array("wrong").is_none());
    }
}
