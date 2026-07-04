//! Select faces within N edge-rings of a seed face.
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

pub fn select_face_ring(mesh: &PolyData, seed_face: usize, rings: usize) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    if seed_face >= nc {
        return PolyData::new();
    }
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, c) in cells.iter().enumerate() {
        let n = c.len();
        for i in 0..n {
            let Ok(a) = usize::try_from(c[i]) else {
                continue;
            };
            let Ok(b) = usize::try_from(c[(i + 1) % n]) else {
                continue;
            };
            if a >= mesh.points.len() || b >= mesh.points.len() {
                continue;
            }
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut fadj: Vec<Vec<usize>> = vec![Vec::new(); nc];
    for (_, faces) in &ef {
        for i in 0..faces.len() {
            for j in i + 1..faces.len() {
                fadj[faces[i]].push(faces[j]);
                fadj[faces[j]].push(faces[i]);
            }
        }
    }
    let mut selected = vec![false; nc];
    selected[seed_face] = true;
    let mut frontier = vec![seed_face];
    for _ in 0..rings {
        let mut next = Vec::new();
        for &fi in &frontier {
            for &ni in &fadj[fi] {
                if !selected[ni] {
                    selected[ni] = true;
                    next.push(ni);
                }
            }
        }
        frontier = next;
    }
    let mut used = vec![false; mesh.points.len()];
    let mut kept = Vec::new();
    let mut old_poly_ids = Vec::new();
    let poly_cell_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    for (ci, c) in cells.iter().enumerate() {
        if selected[ci] {
            let mut valid_vertices = Vec::with_capacity(c.len());
            for &v in c {
                let Ok(v) = usize::try_from(v) else {
                    valid_vertices.clear();
                    break;
                };
                if v >= mesh.points.len() {
                    valid_vertices.clear();
                    break;
                }
                valid_vertices.push(v);
            }
            if valid_vertices.len() != c.len() {
                continue;
            }
            for v in valid_vertices {
                used[v] = true;
            }
            kept.push(c.clone());
            old_poly_ids.push(poly_cell_offset + ci);
        }
    }
    let mut pm = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    let mut original_point_ids = Vec::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pm[i] = pts.len();
            pts.push(mesh.points.get(i));
            original_point_ids.push(i);
        }
    }
    let mut polys = CellArray::new();
    for c in &kept {
        polys.push_cell(
            &c.iter()
                .map(|&v| pm[usize::try_from(v).unwrap()] as i64)
                .collect::<Vec<_>>(),
        );
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    replace_point_data(
        r.point_data_mut(),
        mesh.point_data(),
        &original_point_ids,
        mesh.points.len(),
    );
    remap_cell_data(mesh, &old_poly_ids, &mut r);
    r
}

fn replace_point_data(
    output: &mut DataSetAttributes,
    input: &DataSetAttributes,
    original_ids: &[usize],
    num_input_points: usize,
) {
    let active_scalars = input.scalars().map(|a| a.name().to_string());
    let active_vectors = input.vectors().map(|a| a.name().to_string());
    let active_normals = input.normals().map(|a| a.name().to_string());
    let active_tcoords = input.tcoords().map(|a| a.name().to_string());
    let active_tensors = input.tensors().map(|a| a.name().to_string());
    let active_global_ids = input.global_ids().map(|a| a.name().to_string());
    let active_pedigree_ids = input.pedigree_ids().map(|a| a.name().to_string());
    let active_edge_flags = input.edge_flags().map(|a| a.name().to_string());
    let active_tangents = input.tangents().map(|a| a.name().to_string());
    let active_rational_weights = input.rational_weights().map(|a| a.name().to_string());
    let active_higher_order_degrees = input.higher_order_degrees().map(|a| a.name().to_string());
    let active_process_ids = input.process_ids().map(|a| a.name().to_string());

    output.clear();
    for array in input.field_data().iter() {
        if array.num_tuples() == num_input_points {
            output.add_array(compact_array(array, original_ids));
        }
    }

    if let Some(name) = active_scalars.as_deref() {
        output.set_active_scalars(name);
    }
    if let Some(name) = active_vectors.as_deref() {
        output.set_active_vectors(name);
    }
    if let Some(name) = active_normals.as_deref() {
        output.set_active_normals(name);
    }
    if let Some(name) = active_tcoords.as_deref() {
        output.set_active_tcoords(name);
    }
    if let Some(name) = active_tensors.as_deref() {
        output.set_active_tensors(name);
    }
    if let Some(name) = active_global_ids.as_deref() {
        output.set_active_global_ids(name);
    }
    if let Some(name) = active_pedigree_ids.as_deref() {
        output.set_active_pedigree_ids(name);
    }
    if let Some(name) = active_edge_flags.as_deref() {
        output.set_active_edge_flags(name);
    }
    if let Some(name) = active_tangents.as_deref() {
        output.set_active_tangents(name);
    }
    if let Some(name) = active_rational_weights.as_deref() {
        output.set_active_rational_weights(name);
    }
    if let Some(name) = active_higher_order_degrees.as_deref() {
        output.set_active_higher_order_degrees(name);
    }
    if let Some(name) = active_process_ids.as_deref() {
        output.set_active_process_ids(name);
    }
}

fn compact_array(array: &AnyDataArray, original_ids: &[usize]) -> AnyDataArray {
    macro_rules! compact {
        ($array:expr, $variant:ident) => {{
            let num_components = $array.num_components();
            let mut data = Vec::with_capacity(original_ids.len() * num_components);
            for &source_id in original_ids {
                data.extend_from_slice($array.tuple(source_id));
            }
            AnyDataArray::$variant(DataArray::from_vec($array.name(), data, num_components))
        }};
    }

    match array {
        AnyDataArray::F32(a) => compact!(a, F32),
        AnyDataArray::F64(a) => compact!(a, F64),
        AnyDataArray::I8(a) => compact!(a, I8),
        AnyDataArray::I16(a) => compact!(a, I16),
        AnyDataArray::I32(a) => compact!(a, I32),
        AnyDataArray::I64(a) => compact!(a, I64),
        AnyDataArray::U8(a) => compact!(a, U8),
        AnyDataArray::U16(a) => compact!(a, U16),
        AnyDataArray::U32(a) => compact!(a, U32),
        AnyDataArray::U64(a) => compact!(a, U64),
    }
}

fn remap_cell_data(input: &PolyData, old_poly_ids: &[usize], output: &mut PolyData) {
    if input.cell_data().num_arrays() == 0 {
        return;
    }

    output.cell_data_mut().clear();
    for array in input.cell_data().field_data().iter() {
        if array.num_tuples() == input.total_cells() {
            output
                .cell_data_mut()
                .add_array(remap_array(array, old_poly_ids));
        }
    }
    restore_active_attributes(output.cell_data_mut(), input.cell_data());
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

fn restore_active_attributes(output: &mut DataSetAttributes, input: &DataSetAttributes) {
    if let Some(name) = input.scalars().map(|a| a.name()) {
        output.set_active_scalars(name);
    }
    if let Some(name) = input.vectors().map(|a| a.name()) {
        output.set_active_vectors(name);
    }
    if let Some(name) = input.normals().map(|a| a.name()) {
        output.set_active_normals(name);
    }
    if let Some(name) = input.tcoords().map(|a| a.name()) {
        output.set_active_tcoords(name);
    }
    if let Some(name) = input.tensors().map(|a| a.name()) {
        output.set_active_tensors(name);
    }
    if let Some(name) = input.global_ids().map(|a| a.name()) {
        output.set_active_global_ids(name);
    }
    if let Some(name) = input.pedigree_ids().map(|a| a.name()) {
        output.set_active_pedigree_ids(name);
    }
    if let Some(name) = input.edge_flags().map(|a| a.name()) {
        output.set_active_edge_flags(name);
    }
    if let Some(name) = input.tangents().map(|a| a.name()) {
        output.set_active_tangents(name);
    }
    if let Some(name) = input.rational_weights().map(|a| a.name()) {
        output.set_active_rational_weights(name);
    }
    if let Some(name) = input.higher_order_degrees().map(|a| a.name()) {
        output.set_active_higher_order_degrees(name);
    }
    if let Some(name) = input.process_ids().map(|a| a.name()) {
        output.set_active_process_ids(name);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 4, 3], [1, 3, 2], [2, 3, 5]],
        );
        let r = select_face_ring(&m, 0, 1);
        assert!(r.polys.num_cells() >= 2 && r.polys.num_cells() <= 4);
    }

    #[test]
    fn preserves_point_and_cell_data_for_selected_ring() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "point_id",
                vec![10.0, 11.0, 12.0, 13.0],
                1,
            )));
        m.point_data_mut().set_active_scalars("point_id");
        m.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cell_id",
                vec![20, 21],
                1,
            )));
        m.cell_data_mut().set_active_scalars("cell_id");

        let r = select_face_ring(&m, 0, 0);

        assert_eq!(r.points.len(), 3);
        assert_eq!(r.polys.num_cells(), 1);
        assert_eq!(r.point_data().scalars().unwrap().num_tuples(), 3);
        let cell_id = r.cell_data().scalars().unwrap();
        let mut buf = [0.0];
        cell_id.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 20.0);
    }
}
