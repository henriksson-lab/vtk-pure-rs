use crate::data::PolyData;

/// Create a deep copy of a PolyData, optionally stripping all data arrays.
///
/// `copy_geometry_only` = true: copies points and cells but no point/cell data.
/// `copy_geometry_only` = false: full deep copy (same as clone).
pub fn deep_copy(input: &PolyData, geometry_only: bool) -> PolyData {
    if !geometry_only {
        return input.clone();
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.polys = input.polys.clone();
    pd.lines = input.lines.clone();
    pd.verts = input.verts.clone();
    pd.strips = input.strips.clone();
    pd
}

/// Merge point data from source onto target, matching by point index.
///
/// Copies all arrays from `source` point data to `target` point data,
/// but only if both have the same number of points.
pub fn copy_attributes(source: &PolyData, target: &PolyData) -> PolyData {
    let mut pd = target.clone();
    if source.points.len() != target.points.len() {
        return pd;
    }

    for i in 0..source.point_data().num_arrays() {
        let arr = source.point_data().get_array_by_index(i).unwrap();
        pd.point_data_mut().add_array(arr.clone());
    }
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn geometry_only_strips_data() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![1.0, 2.0, 3.0],
                1,
            )));

        let result = deep_copy(&pd, true);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.point_data().num_arrays(), 0);
    }

    #[test]
    fn full_copy_keeps_data() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", vec![42.0], 1)));

        let result = deep_copy(&pd, false);
        assert!(result.point_data().get_array("val").is_some());
    }

    #[test]
    fn copy_attributes_matching() {
        let mut src = PolyData::new();
        src.points.push([0.0, 0.0, 0.0]);
        src.points.push([1.0, 0.0, 0.0]);
        src.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![100.0, 200.0],
                1,
            )));

        let mut tgt = PolyData::new();
        tgt.points.push([0.0, 0.0, 0.0]);
        tgt.points.push([1.0, 0.0, 0.0]);

        let result = copy_attributes(&src, &tgt);
        assert!(result.point_data().get_array("temp").is_some());
    }

    #[test]
    fn copy_attributes_mismatch() {
        let mut src = PolyData::new();
        src.points.push([0.0, 0.0, 0.0]);
        src.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![100.0],
                1,
            )));

        let mut tgt = PolyData::new();
        tgt.points.push([0.0, 0.0, 0.0]);
        tgt.points.push([1.0, 0.0, 0.0]);

        let result = copy_attributes(&src, &tgt);
        assert!(result.point_data().get_array("temp").is_none());
    }
}
