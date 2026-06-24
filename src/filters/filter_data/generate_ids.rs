use crate::data::{AnyDataArray, DataArray, PolyData};

/// Generate point ID and/or cell ID arrays on a PolyData.
///
/// Adds integer arrays named "PointIds" and/or "CellIds" containing
/// sequential indices starting from 0.
pub fn generate_point_ids(input: &PolyData) -> PolyData {
    let mut pd = input.clone();
    let n = pd.points.len();
    let ids: Vec<i64> = (0..n as i64).collect();
    let arr = AnyDataArray::I64(DataArray::from_vec("PointIds", ids, 1));
    pd.point_data_mut().add_array(arr);
    pd
}

pub fn generate_cell_ids(input: &PolyData) -> PolyData {
    let mut pd = input.clone();
    let n = pd.total_cells();
    let ids: Vec<i64> = (0..n as i64).collect();
    let arr = AnyDataArray::I64(DataArray::from_vec("CellIds", ids, 1));
    pd.cell_data_mut().add_array(arr);
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
        let arr = result.point_data().get_array("PointIds").unwrap();
        assert_eq!(arr.num_tuples(), 3);
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
        let arr = result.cell_data().get_array("CellIds").unwrap();
        assert_eq!(arr.num_tuples(), 2);
    }

    #[test]
    fn both_ids() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = generate_ids(&pd);
        assert!(result.point_data().get_array("PointIds").is_some());
        assert!(result.cell_data().get_array("CellIds").is_some());
    }
}
