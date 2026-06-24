use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Select points by scalar value range and extract with connected cells.
///
/// Keeps points where the named scalar is in [lower, upper], and
/// any cells whose ALL vertices are kept. Point indices are compacted.
pub fn select_by_scalar(input: &PolyData, array_name: &str, lower: f64, upper: f64) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return PolyData::new(),
    };

    let n = input.points.len();
    let mut keep = vec![false; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        keep[i] = buf[0] >= lower && buf[0] <= upper;
    }

    let mut pt_map: HashMap<usize, i64> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.iter().all(|&id| keep[id as usize]) {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    *pt_map.entry(id as usize).or_insert_with(|| {
                        let idx = out_points.len() as i64;
                        out_points.push(input.points.get(id as usize));
                        idx
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn select_range() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]); // scalars: 0,1,2 -> all in [0,2]
        pd.polys.push_cell(&[1, 3, 4]); // scalars: 1,5,4 -> not all in [0,2]
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 1.0, 2.0, 5.0, 4.0],
                1,
            )));

        let result = select_by_scalar(&pd, "s", 0.0, 2.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn select_all() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 1.0, 1.0],
                1,
            )));

        let result = select_by_scalar(&pd, "s", 0.0, 2.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = select_by_scalar(&pd, "nope", 0.0, 1.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
