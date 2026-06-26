use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};
use crate::types::Scalar;

fn dist_sq(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx: f64 = a[0] - b[0];
    let dy: f64 = a[1] - b[1];
    let dz: f64 = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// Triangulate quad faces in a PolyData by splitting along the shorter diagonal.
///
/// Triangles and other non-quad cells are passed through unchanged.
/// Each quad (4-vertex polygon) is split into two triangles along the
/// diagonal that is shorter, producing better-shaped triangles.
pub fn triangulate_quads(input: &PolyData) -> PolyData {
    let mut out_polys = CellArray::new();
    let mut old_poly_ids = Vec::new();
    let poly_cell_offset = input.verts.num_cells() + input.lines.num_cells();

    for (poly_id, cell) in input.polys.iter().enumerate() {
        let old_cell_id = poly_cell_offset + poly_id;
        if cell.len() == 4 {
            let p0 = input.points.get(cell[0] as usize);
            let p1 = input.points.get(cell[1] as usize);
            let p2 = input.points.get(cell[2] as usize);
            let p3 = input.points.get(cell[3] as usize);

            let diag_02: f64 = dist_sq(p0, p2);
            let diag_13: f64 = dist_sq(p1, p3);

            if diag_02 <= diag_13 {
                // Split along 0-2
                out_polys.push_cell(&[cell[0], cell[1], cell[2]]);
                old_poly_ids.push(old_cell_id);
                out_polys.push_cell(&[cell[0], cell[2], cell[3]]);
                old_poly_ids.push(old_cell_id);
            } else {
                // Split along 1-3
                out_polys.push_cell(&[cell[0], cell[1], cell[3]]);
                old_poly_ids.push(old_cell_id);
                out_polys.push_cell(&[cell[1], cell[2], cell[3]]);
                old_poly_ids.push(old_cell_id);
            }
        } else {
            out_polys.push_cell(cell);
            old_poly_ids.push(old_cell_id);
        }
    }

    let mut pd = input.clone();
    pd.polys = out_polys;
    remap_cell_data(input, &old_poly_ids, &mut pd);
    pd
}

fn remap_cell_data(input: &PolyData, old_poly_ids: &[usize], output: &mut PolyData) {
    if input.cell_data().num_arrays() == 0 {
        return;
    }

    let mut old_cell_ids = Vec::with_capacity(output.total_cells());
    old_cell_ids.extend(0..input.verts.num_cells());

    let line_offset = input.verts.num_cells();
    old_cell_ids.extend(line_offset..line_offset + input.lines.num_cells());

    old_cell_ids.extend_from_slice(old_poly_ids);

    let strip_offset = input.verts.num_cells() + input.lines.num_cells() + input.polys.num_cells();
    old_cell_ids.extend(strip_offset..strip_offset + input.strips.num_cells());

    output.cell_data_mut().clear();
    for i in 0..input.cell_data().num_arrays() {
        let Some(array) = input.cell_data().get_array_by_index(i) else {
            continue;
        };
        if array.num_tuples() == input.total_cells() {
            output
                .cell_data_mut()
                .add_array(remap_array(array, &old_cell_ids));
        }
    }
}

fn remap_array(array: &AnyDataArray, old_cell_ids: &[usize]) -> AnyDataArray {
    macro_rules! remap {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(remap_typed_array($array, old_cell_ids))
        };
    }

    match array {
        AnyDataArray::F32(array) => remap!(array, F32),
        AnyDataArray::F64(array) => remap!(array, F64),
        AnyDataArray::I8(array) => remap!(array, I8),
        AnyDataArray::I16(array) => remap!(array, I16),
        AnyDataArray::I32(array) => remap!(array, I32),
        AnyDataArray::I64(array) => remap!(array, I64),
        AnyDataArray::U8(array) => remap!(array, U8),
        AnyDataArray::U16(array) => remap!(array, U16),
        AnyDataArray::U32(array) => remap!(array, U32),
        AnyDataArray::U64(array) => remap!(array, U64),
    }
}

fn remap_typed_array<T: Scalar>(array: &DataArray<T>, old_cell_ids: &[usize]) -> DataArray<T> {
    let mut data = Vec::with_capacity(old_cell_ids.len() * array.num_components());
    for &old_cell_id in old_cell_ids {
        data.extend_from_slice(array.tuple(old_cell_id));
    }
    DataArray::from_vec(array.name(), data, array.num_components())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Points;

    fn make_quad() -> PolyData {
        let mut pts = Points::<f64>::new();
        pts.push([0.0, 0.0, 0.0]);
        pts.push([1.0, 0.0, 0.0]);
        pts.push([1.0, 1.0, 0.0]);
        pts.push([0.0, 1.0, 0.0]);

        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2, 3]);

        let mut pd = PolyData::new();
        pd.points = pts;
        pd.polys = polys;
        pd
    }

    #[test]
    fn quad_becomes_two_triangles() {
        let input = make_quad();
        let result = triangulate_quads(&input);
        assert_eq!(result.polys.num_cells(), 2);
        // Points should be the same (shared)
        assert_eq!(result.points.len(), 4);
        // Each cell should have 3 vertices
        for cell in result.polys.iter() {
            assert_eq!(cell.len(), 3);
        }
    }

    #[test]
    fn triangles_pass_through() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = triangulate_quads(&pd);
        assert_eq!(result.polys.num_cells(), 1);
        let cell: Vec<i64> = result.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell.len(), 3);
    }

    #[test]
    fn shorter_diagonal_split() {
        // Make a non-square quad where diagonal 0-2 is shorter than 1-3
        let mut pts = Points::<f64>::new();
        pts.push([0.0, 0.0, 0.0]);
        pts.push([2.0, 0.0, 0.0]);
        pts.push([0.5, 0.5, 0.0]); // close to vertex 0
        pts.push([0.0, 2.0, 0.0]);

        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2, 3]);

        let mut pd = PolyData::new();
        pd.points = pts;
        pd.polys = polys;

        let result = triangulate_quads(&pd);
        assert_eq!(result.polys.num_cells(), 2);

        // Diagonal 0-2 (distance sqrt(0.5)) is shorter than 1-3 (distance sqrt(8))
        // so should split along 0-2: triangles (0,1,2) and (0,2,3)
        let cells: Vec<Vec<i64>> = result.polys.iter().map(|c| c.to_vec()).collect();
        assert!(cells[0].contains(&0) && cells[0].contains(&1) && cells[0].contains(&2));
        assert!(cells[1].contains(&0) && cells[1].contains(&2) && cells[1].contains(&3));
    }

    #[test]
    fn point_and_cell_data_pass_through() {
        use crate::data::{AnyDataArray, DataArray};

        let mut input = make_quad();
        input
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "point_id",
                vec![0.0, 1.0, 2.0, 3.0],
                1,
            )));
        input
            .cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "cell_id",
                vec![9.0],
                1,
            )));

        let result = triangulate_quads(&input);

        assert!(result.point_data().get_array("point_id").is_some());
        let cell_id = result.cell_data().get_array("cell_id").unwrap();
        assert_eq!(cell_id.num_tuples(), result.total_cells());
        let mut buf = [0.0];
        cell_id.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 9.0);
        cell_id.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 9.0);
    }
}
