use crate::data::{DataSetAttributes, FieldData, PolyData};

/// Select which arrays to keep in point, cell, and field data.
///
/// Removes all arrays except those whose names are in `keep_names`.
/// If `keep_names` is empty, removes all non-ghost arrays.
pub fn pass_arrays(input: &PolyData, keep_names: &[&str]) -> PolyData {
    let mut pd = input.clone();

    filter_attributes(pd.point_data_mut(), keep_names);
    filter_attributes(pd.cell_data_mut(), keep_names);
    filter_field_data(pd.field_data_mut(), keep_names);

    pd
}

/// Remove specific arrays by name from point, cell, and field data.
pub fn remove_arrays(input: &PolyData, remove_names: &[&str]) -> PolyData {
    let mut pd = input.clone();

    remove_from_attributes(pd.point_data_mut(), remove_names);
    remove_from_attributes(pd.cell_data_mut(), remove_names);
    remove_from_field_data(pd.field_data_mut(), remove_names);

    pd
}

fn filter_attributes(attrs: &mut DataSetAttributes, keep_names: &[&str]) {
    let mut remove_names = Vec::new();
    for i in 0..attrs.num_arrays() {
        let a = attrs.get_array_by_index(i).unwrap();
        if !keep_names.contains(&a.name()) && !is_ghost_array(a.name()) {
            remove_names.push(a.name().to_string());
        }
    }
    for name in remove_names {
        attrs.remove_array(&name);
    }
}

fn remove_from_attributes(attrs: &mut DataSetAttributes, remove_names: &[&str]) {
    for name in remove_names {
        attrs.remove_array(name);
    }
}

fn filter_field_data(attrs: &mut FieldData, keep_names: &[&str]) {
    let mut remove_names = Vec::new();
    for a in attrs.iter() {
        if !keep_names.contains(&a.name()) && !is_ghost_array(a.name()) {
            remove_names.push(a.name().to_string());
        }
    }
    for name in remove_names {
        attrs.remove_array(&name);
    }
}

fn remove_from_field_data(attrs: &mut FieldData, remove_names: &[&str]) {
    for name in remove_names {
        attrs.remove_array(name);
    }
}

fn is_ghost_array(name: &str) -> bool {
    name == "vtkGhostType" || name == "GhostType"
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
        pd.field_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("meta", vec![7.0], 1)));
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
        assert_eq!(result.field_data().num_arrays(), 0);
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
        assert_eq!(result.field_data().num_arrays(), 1);
    }

    #[test]
    fn keep_field_data_by_name() {
        let pd = make_test_pd();
        let result = pass_arrays(&pd, &["meta"]);
        assert!(result.field_data().get_array("meta").is_some());
    }

    #[test]
    fn pass_mode_preserves_ghost_arrays() {
        let mut pd = make_test_pd();
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "vtkGhostType",
                vec![0.0],
                1,
            )));

        let result = pass_arrays(&pd, &[]);
        assert!(result.point_data().get_array("vtkGhostType").is_some());
    }

    #[test]
    fn pass_mode_preserves_active_attribute_for_kept_array() {
        let mut pd = make_test_pd();
        pd.point_data_mut().set_active_scalars("scalars");

        let result = pass_arrays(&pd, &["scalars"]);

        assert!(result.point_data().scalars().is_some());
        assert!(result.point_data().get_array("temp").is_none());
    }

    #[test]
    fn remove_mode_preserves_active_attribute_for_unremoved_array() {
        let mut pd = make_test_pd();
        pd.point_data_mut().set_active_scalars("scalars");

        let result = remove_arrays(&pd, &["temp"]);

        assert!(result.point_data().scalars().is_some());
        assert!(result.point_data().get_array("temp").is_none());
    }
}
