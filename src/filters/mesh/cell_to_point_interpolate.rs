use crate::data::{AnyDataArray, DataArray, PolyData};

/// Interpolate cell data to point data using area-weighted averaging.
///
/// Unlike simple cell_data_to_point_data which uses uniform weights,
/// this weights each cell's contribution by its area relative to
/// the total area of cells sharing that vertex.
pub fn cell_to_point_area_weighted(input: &PolyData, array_name: &str) -> PolyData {
    let arr = match input.cell_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = input.points.len();
    let nc = arr.num_components();
    let mut sums = vec![0.0f64; n * nc];
    let mut weights = vec![0.0f64; n];
    let mut buf = vec![0.0f64; nc];

    for (ci, cell) in input.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        arr.tuple_as_f64(ci, &mut buf);

        // Compute cell area
        let v0 = input.points.get(cell[0] as usize);
        let mut area = 0.0;
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let cx = e1[1] * e2[2] - e1[2] * e2[1];
            let cy = e1[2] * e2[0] - e1[0] * e2[2];
            let cz = e1[0] * e2[1] - e1[1] * e2[0];
            area += 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
        }

        for &pid in cell.iter() {
            let idx = pid as usize;
            weights[idx] += area;
            for c in 0..nc {
                sums[idx * nc + c] += buf[c] * area;
            }
        }
    }

    // Normalize
    for i in 0..n {
        if weights[i] > 1e-15 {
            for c in 0..nc {
                sums[i * nc + c] /= weights[i];
            }
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, sums, nc)));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_weighted_interpolation() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![10.0, 20.0],
                1,
            )));

        let result = cell_to_point_area_weighted(&pd, "temp");
        let arr = result.point_data().get_array("temp").unwrap();
        let mut buf = [0.0f64];
        // Point 0 and 2 shared by both cells with equal area
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 15.0).abs() < 1e-10);
    }

    #[test]
    fn single_cell() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", vec![42.0], 1)));

        let result = cell_to_point_area_weighted(&pd, "val");
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 42.0);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = cell_to_point_area_weighted(&pd, "nope");
        assert_eq!(result.points.len(), 0);
    }
}
