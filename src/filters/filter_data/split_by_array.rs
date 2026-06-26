use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Split a PolyData into multiple PolyData based on a cell data array.
///
/// Groups cells by their integer value in the named cell data array.
/// Returns a Vec of (value, PolyData) pairs, one per unique value.
pub fn split_by_array(input: &PolyData, array_name: &str) -> Vec<(i64, PolyData)> {
    let arr = match input.cell_data().get_array(array_name) {
        Some(a) => a,
        None => return vec![],
    };

    // Group cell indices by array value
    let mut groups: HashMap<i64, Vec<usize>> = HashMap::new();
    let mut buf = [0.0f64];
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();

    let num_values = arr.num_tuples().min(cells.len());
    for ci in 0..num_values {
        arr.tuple_as_f64(ci, &mut buf);
        let key = buf[0] as i64;
        groups.entry(key).or_default().push(ci);
    }

    let mut result: Vec<(i64, PolyData)> = Vec::new();

    for (value, cell_indices) in &groups {
        let mut pt_map: HashMap<i64, i64> = HashMap::new();
        let mut out_points = Points::<f64>::new();
        let mut out_polys = CellArray::new();

        for &ci in cell_indices {
            let cell = &cells[ci];
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&pid| {
                    *pt_map.entry(pid).or_insert_with(|| {
                        let idx = out_points.len() as i64;
                        out_points.push(input.points.get(pid as usize));
                        idx
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }

        let mut pd = PolyData::new();
        pd.points = out_points;
        pd.polys = out_polys;
        result.push((*value, pd));
    }

    result.sort_by_key(|&(v, _)| v);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn split_by_region() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([3.0, 0.0, 0.0]);
        pd.points.push([2.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "region",
                vec![0.0, 1.0],
                1,
            )));

        let parts = split_by_array(&pd, "region");
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].0, 0);
        assert_eq!(parts[0].1.polys.num_cells(), 1);
        assert_eq!(parts[1].0, 1);
        assert_eq!(parts[1].1.polys.num_cells(), 1);
    }

    #[test]
    fn all_same_group() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("g", vec![5.0], 1)));

        let parts = split_by_array(&pd, "g");
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].0, 5);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let parts = split_by_array(&pd, "nope");
        assert_eq!(parts.len(), 0);
    }
}
