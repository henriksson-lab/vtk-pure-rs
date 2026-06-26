use crate::data::{DataSetAttributes, PolyData};

/// Rename a point or cell data array.
///
/// If `from_name` is found in point data or cell data, renames it to `to_name`.
/// Returns a new PolyData with the renamed array.
pub fn array_rename(input: &PolyData, from_name: &str, to_name: &str) -> PolyData {
    let mut pd = input.clone();

    rename_in_attributes(pd.point_data_mut(), from_name, to_name);
    rename_in_attributes(pd.cell_data_mut(), from_name, to_name);

    pd
}

fn rename_in_attributes(attrs: &mut DataSetAttributes, from: &str, to: &str) {
    if let Some(arr) = attrs.get_array(from) {
        let mut renamed = arr.clone();
        renamed.set_name(to);
        let mut new_attrs = DataSetAttributes::new();
        for i in 0..attrs.num_arrays() {
            let a = attrs.get_array_by_index(i).unwrap();
            if a.name() == from {
                new_attrs.add_array(renamed.clone());
            } else {
                new_attrs.add_array(a.clone());
            }
        }
        *attrs = new_attrs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn rename_point_data() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "old_name",
                vec![1.0],
                1,
            )));

        let result = array_rename(&pd, "old_name", "new_name");
        assert!(result.point_data().get_array("new_name").is_some());
        assert!(result.point_data().get_array("old_name").is_none());
    }

    #[test]
    fn rename_cell_data() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("area", vec![0.5], 1)));

        let result = array_rename(&pd, "area", "cell_area");
        assert!(result.cell_data().get_array("cell_area").is_some());
        assert!(result.cell_data().get_array("area").is_none());
    }

    #[test]
    fn nonexistent_name_noop() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("foo", vec![1.0], 1)));

        let result = array_rename(&pd, "bar", "baz");
        assert!(result.point_data().get_array("foo").is_some());
    }
}
