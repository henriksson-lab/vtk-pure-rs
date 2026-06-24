use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Select cells where ALL vertices satisfy a scalar predicate.
///
/// Keeps cells where `predicate(scalar_value)` is true for every vertex.
/// Compacts point indices. Generic version of threshold/select filters.
pub fn select_cells_by_predicate<F: Fn(f64) -> bool>(
    input: &PolyData,
    array_name: &str,
    predicate: F,
) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let n = input.points.len();
    let mut buf = [0.0f64];
    let keep: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            predicate(buf[0])
        })
        .collect();

    let mut pt_map: HashMap<i64, i64> = HashMap::new();
    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.iter().all(|&id| keep[id as usize]) {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    *pt_map.entry(id).or_insert_with(|| {
                        let i = out_pts.len() as i64;
                        out_pts.push(input.points.get(id as usize));
                        i
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    pd
}

/// Select cells where ANY vertex satisfies the predicate.
pub fn select_cells_any_vertex<F: Fn(f64) -> bool>(
    input: &PolyData,
    array_name: &str,
    predicate: F,
) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let n = input.points.len();
    let mut buf = [0.0f64];
    let vals: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            predicate(buf[0])
        })
        .collect();

    let mut pt_map: HashMap<i64, i64> = HashMap::new();
    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.iter().any(|&id| vals[id as usize]) {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    *pt_map.entry(id).or_insert_with(|| {
                        let i = out_pts.len() as i64;
                        out_pts.push(input.points.get(id as usize));
                        i
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn select_all_above() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![5.0, 5.0, 5.0, 1.0],
                1,
            )));

        let result = select_cells_by_predicate(&pd, "v", |v| v > 3.0);
        assert_eq!(result.polys.num_cells(), 1); // only first cell
    }

    #[test]
    fn select_any_above() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![5.0, 5.0, 5.0, 1.0],
                1,
            )));

        let result = select_cells_any_vertex(&pd, "v", |v| v > 3.0);
        assert_eq!(result.polys.num_cells(), 2); // both have at least one vertex > 3
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        assert_eq!(
            select_cells_by_predicate(&pd, "nope", |_| true)
                .polys
                .num_cells(),
            0
        );
    }
}
