use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Convert cell data to face-varying (per-face-vertex) point data.
///
/// Duplicates vertices so each face has its own copy, then converts
/// cell data to point data. This enables per-face colors/attributes
/// in renderers that only support per-vertex data.
pub fn cell_data_to_face_varying(input: &PolyData, array_name: &str) -> PolyData {
    let arr = match input.cell_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let nc = arr.num_components();
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();
    let mut out_values = Vec::new();
    let mut buf = vec![0.0f64; nc];

    for (ci, cell) in input.polys.iter().enumerate() {
        arr.tuple_as_f64(ci, &mut buf);
        let base = out_points.len() as i64;
        let mut ids = Vec::with_capacity(cell.len());
        for &pid in cell.iter() {
            out_points.push(input.points.get(pid as usize));
            for c in 0..nc {
                out_values.push(buf[c]);
            }
            ids.push(base + ids.len() as i64);
        }
        out_polys.push_cell(&ids);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, out_values, nc,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_color_to_vertex() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "color",
                vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                3,
            )));

        let result = cell_data_to_face_varying(&pd, "color");
        assert_eq!(result.points.len(), 6); // 3+3 (no sharing)
        assert!(result.point_data().get_array("color").is_some());
    }

    #[test]
    fn scalar_cell_data() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![100.0],
                1,
            )));

        let result = cell_data_to_face_varying(&pd, "temp");
        let arr = result.point_data().get_array("temp").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 100.0);
        }
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = cell_data_to_face_varying(&pd, "nope");
        assert_eq!(result.points.len(), 0);
    }
}
