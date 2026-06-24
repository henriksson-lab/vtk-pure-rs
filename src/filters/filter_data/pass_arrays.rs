use crate::data::{DataSetAttributes, PolyData};

/// Select which arrays to keep in point and cell data.
///
/// Removes all arrays except those whose names are in `keep_names`.
/// If `keep_names` is empty, removes all arrays.
pub fn pass_arrays(input: &PolyData, keep_names: &[&str]) -> PolyData {
    let mut pd = input.clone();

    filter_attributes(pd.point_data_mut(), keep_names);
    filter_attributes(pd.cell_data_mut(), keep_names);

    pd
}

/// Remove specific arrays by name.
pub fn remove_arrays(input: &PolyData, remove_names: &[&str]) -> PolyData {
    let mut pd = input.clone();

    remove_from_attributes(pd.point_data_mut(), remove_names);
    remove_from_attributes(pd.cell_data_mut(), remove_names);

    pd
}

fn filter_attributes(attrs: &mut DataSetAttributes, keep_names: &[&str]) {
    let mut new_attrs = DataSetAttributes::new();
    for i in 0..attrs.num_arrays() {
        let a = attrs.get_array_by_index(i).unwrap();
        if keep_names.contains(&a.name()) {
            new_attrs.add_array(a.clone());
        }
    }
    *attrs = new_attrs;
}

fn remove_from_attributes(attrs: &mut DataSetAttributes, remove_names: &[&str]) {
    let mut new_attrs = DataSetAttributes::new();
    for i in 0..attrs.num_arrays() {
        let a = attrs.get_array_by_index(i).unwrap();
        if !remove_names.contains(&a.name()) {
            new_attrs.add_array(a.clone());
        }
    }
    *attrs = new_attrs;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    fn make_test_pd() -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "scalars",
                vec![1.0],
                1,
            )));
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "normals",
                vec![0.0, 0.0, 1.0],
                3,
            )));
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![37.0],
                1,
            )));
        pd
    }

    #[test]
    fn keep_specific() {
        let pd = make_test_pd();
        let result = pass_arrays(&pd, &["scalars", "normals"]);
        assert!(result.point_data().get_array("scalars").is_some());
        assert!(result.point_data().get_array("normals").is_some());
        assert!(result.point_data().get_array("temp").is_none());
    }

    #[test]
    fn keep_none() {
        let pd = make_test_pd();
        let result = pass_arrays(&pd, &[]);
        assert_eq!(result.point_data().num_arrays(), 0);
    }

    #[test]
    fn remove_specific() {
        let pd = make_test_pd();
        let result = remove_arrays(&pd, &["temp"]);
        assert!(result.point_data().get_array("scalars").is_some());
        assert!(result.point_data().get_array("normals").is_some());
        assert!(result.point_data().get_array("temp").is_none());
    }

    #[test]
    fn remove_nonexistent() {
        let pd = make_test_pd();
        let result = remove_arrays(&pd, &["missing"]);
        assert_eq!(result.point_data().num_arrays(), 3);
    }
}
