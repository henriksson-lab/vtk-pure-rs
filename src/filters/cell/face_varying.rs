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
    let mut point_ids = Vec::new();
    let mut out_values = Vec::new();
    let mut buf = vec![0.0f64; nc];

    for (ci, cell) in input.polys.iter().enumerate() {
        arr.tuple_as_f64(ci, &mut buf);
        let base = out_points.len() as i64;
        let mut ids = Vec::with_capacity(cell.len());
        for &pid in cell.iter() {
            out_points.push(input.points.get(pid as usize));
            point_ids.push(pid as usize);
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
    duplicate_point_data(input, &mut pd, &point_ids);
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, out_values, nc,
        )));
    pd
}

fn duplicate_point_data(input: &PolyData, output: &mut PolyData, point_ids: &[usize]) {
    for array_index in 0..input.point_data().num_arrays() {
        if let Some(array) = input.point_data().get_array_by_index(array_index) {
            output
                .point_data_mut()
                .add_array(duplicate_array(array, point_ids));
        }
    }
}

fn duplicate_array(array: &AnyDataArray, point_ids: &[usize]) -> AnyDataArray {
    macro_rules! duplicate_variant {
        ($array:expr, $variant:ident) => {{
            let mut duplicated = DataArray::new($array.name(), $array.num_components());
            for &point_id in point_ids {
                duplicated.push_tuple($array.tuple(point_id));
            }
            AnyDataArray::$variant(duplicated)
        }};
    }

    match array {
        AnyDataArray::F32(array) => duplicate_variant!(array, F32),
        AnyDataArray::F64(array) => duplicate_variant!(array, F64),
        AnyDataArray::I8(array) => duplicate_variant!(array, I8),
        AnyDataArray::I16(array) => duplicate_variant!(array, I16),
        AnyDataArray::I32(array) => duplicate_variant!(array, I32),
        AnyDataArray::I64(array) => duplicate_variant!(array, I64),
        AnyDataArray::U8(array) => duplicate_variant!(array, U8),
        AnyDataArray::U16(array) => duplicate_variant!(array, U16),
        AnyDataArray::U32(array) => duplicate_variant!(array, U32),
        AnyDataArray::U64(array) => duplicate_variant!(array, U64),
    }
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
    fn preserves_point_data_on_duplicated_vertices() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "pid",
                vec![10.0, 11.0, 12.0, 13.0],
                1,
            )));
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "cell",
                vec![20.0, 21.0],
                1,
            )));

        let result = cell_data_to_face_varying(&pd, "cell");
        let arr = result.point_data().get_array("pid").unwrap();
        assert_eq!(arr.to_f64_vec(), vec![10.0, 11.0, 12.0, 10.0, 12.0, 13.0]);
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
