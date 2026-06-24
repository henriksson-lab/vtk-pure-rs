use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute edge lengths for each polygon cell and add as cell data.
///
/// Adds a "MinEdgeLength", "MaxEdgeLength", and "MeanEdgeLength" scalar
/// arrays to cell data.
pub fn compute_edge_lengths(input: &PolyData) -> PolyData {
    let mut min_lengths = Vec::new();
    let mut max_lengths = Vec::new();
    let mut mean_lengths = Vec::new();

    for cell in input.polys.iter() {
        let n = cell.len();
        if n < 2 {
            min_lengths.push(0.0);
            max_lengths.push(0.0);
            mean_lengths.push(0.0);
            continue;
        }

        let mut min_l = f64::MAX;
        let mut max_l = 0.0f64;
        let mut sum_l = 0.0;

        for i in 0..n {
            let a = input.points.get(cell[i] as usize);
            let b = input.points.get(cell[(i + 1) % n] as usize);
            let d = ((a[0] - b[0]) * (a[0] - b[0])
                + (a[1] - b[1]) * (a[1] - b[1])
                + (a[2] - b[2]) * (a[2] - b[2]))
                .sqrt();
            min_l = min_l.min(d);
            max_l = max_l.max(d);
            sum_l += d;
        }

        min_lengths.push(min_l);
        max_lengths.push(max_l);
        mean_lengths.push(sum_l / n as f64);
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MinEdgeLength",
            min_lengths,
            1,
        )));
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MaxEdgeLength",
            max_lengths,
            1,
        )));
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MeanEdgeLength",
            mean_lengths,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equilateral_triangle() {
        let s = 3.0f64.sqrt();
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, s / 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_edge_lengths(&pd);
        let min_arr = result.cell_data().get_array("MinEdgeLength").unwrap();
        let max_arr = result.cell_data().get_array("MaxEdgeLength").unwrap();
        let mut min_val = [0.0f64];
        let mut max_val = [0.0f64];
        min_arr.tuple_as_f64(0, &mut min_val);
        max_arr.tuple_as_f64(0, &mut max_val);
        assert!((min_val[0] - max_val[0]).abs() < 1e-10); // all edges equal
    }

    #[test]
    fn right_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 4.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_edge_lengths(&pd);
        let max_arr = result.cell_data().get_array("MaxEdgeLength").unwrap();
        let mut val = [0.0f64];
        max_arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 5.0).abs() < 1e-10); // hypotenuse = 5
    }
}
