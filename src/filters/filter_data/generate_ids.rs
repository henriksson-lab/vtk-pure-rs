use crate::data::{AnyDataArray, DataArray, PolyData};

/// Generate point ID and/or cell ID arrays on a PolyData.
///
/// Adds integer arrays named "vtkPointIds" and/or "vtkCellIds" containing
/// sequential indices starting from 0.
pub fn generate_point_ids(input: &PolyData) -> PolyData {
    let mut pd = input.clone();
    let n = pd.points.len();
    if n == 0 {
        return pd;
    }
    let ids: Vec<i64> = (0..n as i64).collect();
    let arr = AnyDataArray::I64(DataArray::from_vec("vtkPointIds", ids, 1));
    pd.point_data_mut().add_array(arr);
    pd.point_data_mut().set_active_scalars("vtkPointIds");
    pd
}

pub fn generate_cell_ids(input: &PolyData) -> PolyData {
    let mut pd = input.clone();
    let n = pd.total_cells();
    if n == 0 {
        return pd;
    }
    let ids: Vec<i64> = (0..n as i64).collect();
    let arr = AnyDataArray::I64(DataArray::from_vec("vtkCellIds", ids, 1));
    pd.cell_data_mut().add_array(arr);
    pd.cell_data_mut().set_active_scalars("vtkCellIds");
    pd
}

/// Generate both point and cell ID arrays.
pub fn generate_ids(input: &PolyData) -> PolyData {
    let pd = generate_point_ids(input);
    generate_cell_ids(&pd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_ids() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = generate_point_ids(&pd);
        let arr = result.point_data().get_array("vtkPointIds").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        assert!(result.point_data().scalars().is_some());
        let mut val = [0.0f64];
        arr.tuple_as_f64(2, &mut val);
        assert_eq!(val[0], 2.0);
    }

    #[test]
    fn cell_ids() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = generate_cell_ids(&pd);
        let arr = result.cell_data().get_array("vtkCellIds").unwrap();
        assert_eq!(arr.num_tuples(), 2);
        assert!(result.cell_data().scalars().is_some());
    }

    #[test]
    fn both_ids() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = generate_ids(&pd);
        assert!(result.point_data().get_array("vtkPointIds").is_some());
        assert!(result.cell_data().get_array("vtkCellIds").is_some());
    }

    #[test]
    fn empty_input_does_not_add_arrays() {
        let pd = PolyData::new();
        let result = generate_ids(&pd);
        assert_eq!(result.point_data().num_arrays(), 0);
        assert_eq!(result.cell_data().num_arrays(), 0);
    }
}
