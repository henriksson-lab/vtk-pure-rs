//! Reverse the winding order of all faces (flip normals).
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, PolyData};

pub fn flip_normals(mesh: &PolyData) -> PolyData {
    let mut result = mesh.clone();
    result.verts = reverse_cell_array(&mesh.verts);
    result.lines = reverse_cell_array(&mesh.lines);
    result.polys = reverse_cell_array(&mesh.polys);
    result.strips = reverse_cell_array(&mesh.strips);
    if let Some(flipped) = flipped_normals(normals_array(result.point_data())) {
        result.point_data_mut().add_array(flipped);
    }
    if let Some(flipped) = flipped_normals(normals_array(result.cell_data())) {
        result.cell_data_mut().add_array(flipped);
    }
    result
}

fn reverse_cell_array(cells: &CellArray) -> CellArray {
    let mut reversed_cells = CellArray::new();
    for cell in cells.iter() {
        let mut reversed: Vec<i64> = cell.to_vec();
        reversed.reverse();
        reversed_cells.push_cell(&reversed);
    }
    reversed_cells
}

fn normals_array(attrs: &DataSetAttributes) -> Option<&AnyDataArray> {
    attrs.normals().or_else(|| attrs.get_array("Normals"))
}

fn flipped_normals(normals: Option<&AnyDataArray>) -> Option<AnyDataArray> {
    let normals = normals?;
    if normals.num_components() != 3 {
        return None;
    }

    macro_rules! flip_typed {
        ($array:expr, $variant:ident) => {{
            let data = $array
                .as_slice()
                .iter()
                .map(|&value| crate::types::Scalar::from_f64(-crate::types::Scalar::to_f64(value)))
                .collect();
            Some(AnyDataArray::$variant(DataArray::from_vec(
                $array.name(),
                data,
                3,
            )))
        }};
    }

    match normals {
        AnyDataArray::F32(array) => flip_typed!(array, F32),
        AnyDataArray::F64(array) => flip_typed!(array, F64),
        AnyDataArray::I8(array) => flip_typed!(array, I8),
        AnyDataArray::I16(array) => flip_typed!(array, I16),
        AnyDataArray::I32(array) => flip_typed!(array, I32),
        AnyDataArray::I64(array) => flip_typed!(array, I64),
        AnyDataArray::U8(array) => flip_typed!(array, U8),
        AnyDataArray::U16(array) => flip_typed!(array, U16),
        AnyDataArray::U32(array) => flip_typed!(array, U32),
        AnyDataArray::U64(array) => flip_typed!(array, U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_flip() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = flip_normals(&mesh);
        let cell: Vec<i64> = r.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell, vec![2, 1, 0]);
    }

    #[test]
    fn test_flip_normals_array() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                3,
            )));
        let r = flip_normals(&mesh);
        let mut buf = [0.0f64; 3];
        r.point_data()
            .get_array("Normals")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [0.0, 0.0, -1.0]);
    }
}
