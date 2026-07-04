//! Convert vertex data to face data by averaging, and vice versa.
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, PolyData};
use crate::types::Scalar;
pub fn vertex_to_face_average(mesh: &PolyData, array_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(average_points_onto_cells(mesh, arr, array_name));
    copy_matching_active_attributes(mesh.point_data(), r.cell_data_mut(), array_name);
    r
}

pub fn vertex_to_face_average_all(mesh: &PolyData) -> PolyData {
    let mut r = mesh.clone();
    for arr in mesh.point_data().iter() {
        if arr.num_tuples() == mesh.points.len() {
            r.cell_data_mut()
                .add_array(average_points_onto_cells(mesh, arr, arr.name()));
        }
    }
    copy_active_attributes(mesh.point_data(), r.cell_data_mut());
    r
}

fn average_points_onto_cells(
    mesh: &PolyData,
    arr: &AnyDataArray,
    array_name: &str,
) -> AnyDataArray {
    macro_rules! average {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(average_points_onto_cells_typed(mesh, $array, array_name))
        };
    }

    match arr {
        AnyDataArray::F32(a) => average!(a, F32),
        AnyDataArray::F64(a) => average!(a, F64),
        AnyDataArray::I8(a) => average!(a, I8),
        AnyDataArray::I16(a) => average!(a, I16),
        AnyDataArray::I32(a) => average!(a, I32),
        AnyDataArray::I64(a) => average!(a, I64),
        AnyDataArray::U8(a) => average!(a, U8),
        AnyDataArray::U16(a) => average!(a, U16),
        AnyDataArray::U32(a) => average!(a, U32),
        AnyDataArray::U64(a) => average!(a, U64),
    }
}

fn average_points_onto_cells_typed<T: Scalar>(
    mesh: &PolyData,
    arr: &DataArray<T>,
    array_name: &str,
) -> DataArray<T> {
    let nc = arr.num_components();
    let nt = mesh.points.len().min(arr.num_tuples());
    let mut data = Vec::with_capacity(mesh.total_cells() * nc);
    append_vertex_to_face_average(&mesh.verts, arr, nt, nc, &mut data);
    append_vertex_to_face_average(&mesh.lines, arr, nt, nc, &mut data);
    append_vertex_to_face_average(&mesh.polys, arr, nt, nc, &mut data);
    append_vertex_to_face_average(&mesh.strips, arr, nt, nc, &mut data);
    DataArray::from_vec(array_name, data, nc)
}

fn append_vertex_to_face_average<T: Scalar>(
    cells: &CellArray,
    arr: &DataArray<T>,
    nt: usize,
    nc: usize,
    data: &mut Vec<T>,
) {
    for cell in cells.iter() {
        let valid: Vec<usize> = cell
            .iter()
            .filter_map(|&v| {
                let vi = usize::try_from(v).ok()?;
                (vi < nt).then_some(vi)
            })
            .collect();
        let nv = valid.len();
        if nv == 0 {
            for _ in 0..nc {
                data.push(T::from_f64(0.0));
            }
            continue;
        }
        let mut avg = vec![0.0f64; nc];
        for &v in &valid {
            let tuple = arr.tuple(v);
            for c in 0..nc {
                avg[c] += tuple[c].to_f64();
            }
        }
        for c in 0..nc {
            data.push(T::from_f64(avg[c] / nv as f64));
        }
    }
}
pub fn face_to_vertex_average(mesh: &PolyData, array_name: &str) -> PolyData {
    let arr = match mesh.cell_data().get_array(array_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(average_cells_onto_points(mesh, arr, array_name));
    copy_matching_active_attributes(mesh.cell_data(), r.point_data_mut(), array_name);
    r
}

pub fn face_to_vertex_average_all(mesh: &PolyData) -> PolyData {
    let mut r = mesh.clone();
    for arr in mesh.cell_data().iter() {
        if arr.num_tuples() == mesh.total_cells() {
            r.point_data_mut()
                .add_array(average_cells_onto_points(mesh, arr, arr.name()));
        }
    }
    copy_active_attributes(mesh.cell_data(), r.point_data_mut());
    r
}

fn average_cells_onto_points(
    mesh: &PolyData,
    arr: &AnyDataArray,
    array_name: &str,
) -> AnyDataArray {
    macro_rules! average {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(average_cells_onto_points_typed(mesh, $array, array_name))
        };
    }

    match arr {
        AnyDataArray::F32(a) => average!(a, F32),
        AnyDataArray::F64(a) => average!(a, F64),
        AnyDataArray::I8(a) => average!(a, I8),
        AnyDataArray::I16(a) => average!(a, I16),
        AnyDataArray::I32(a) => average!(a, I32),
        AnyDataArray::I64(a) => average!(a, I64),
        AnyDataArray::U8(a) => average!(a, U8),
        AnyDataArray::U16(a) => average!(a, U16),
        AnyDataArray::U32(a) => average!(a, U32),
        AnyDataArray::U64(a) => average!(a, U64),
    }
}

fn average_cells_onto_points_typed<T: Scalar>(
    mesh: &PolyData,
    arr: &DataArray<T>,
    array_name: &str,
) -> DataArray<T> {
    let nc = arr.num_components();
    let npts = mesh.points.len();
    let mut sums = vec![0.0f64; npts * nc];
    let mut counts = vec![0usize; npts];
    let mut cell_id = 0usize;
    accumulate_face_to_vertex_average(
        &mesh.verts,
        arr,
        npts,
        nc,
        &mut cell_id,
        &mut sums,
        &mut counts,
    );
    accumulate_face_to_vertex_average(
        &mesh.lines,
        arr,
        npts,
        nc,
        &mut cell_id,
        &mut sums,
        &mut counts,
    );
    accumulate_face_to_vertex_average(
        &mesh.polys,
        arr,
        npts,
        nc,
        &mut cell_id,
        &mut sums,
        &mut counts,
    );
    accumulate_face_to_vertex_average(
        &mesh.strips,
        arr,
        npts,
        nc,
        &mut cell_id,
        &mut sums,
        &mut counts,
    );
    let mut data = Vec::with_capacity(npts * nc);
    for i in 0..npts {
        for c in 0..nc {
            let value = if counts[i] > 0 {
                sums[i * nc + c] / counts[i] as f64
            } else {
                0.0
            };
            data.push(T::from_f64(value));
        }
    }
    DataArray::from_vec(array_name, data, nc)
}

fn accumulate_face_to_vertex_average<T: Scalar>(
    cells: &CellArray,
    arr: &DataArray<T>,
    npts: usize,
    nc: usize,
    cell_id: &mut usize,
    sums: &mut [f64],
    counts: &mut [usize],
) {
    for cell in cells.iter() {
        if *cell_id < arr.num_tuples() {
            let tuple = arr.tuple(*cell_id);
            for &v in cell {
                if let Some(vi) = usize::try_from(v).ok().filter(|&vi| vi < npts) {
                    counts[vi] += 1;
                    for c in 0..nc {
                        sums[vi * nc + c] += tuple[c].to_f64();
                    }
                }
            }
        }
        *cell_id += 1;
    }
}

fn copy_matching_active_attributes(
    input: &DataSetAttributes,
    output: &mut DataSetAttributes,
    array_name: &str,
) {
    if input
        .scalars()
        .is_some_and(|array| array.name() == array_name)
    {
        output.set_active_scalars(array_name);
    }
    if input
        .vectors()
        .is_some_and(|array| array.name() == array_name)
    {
        output.set_active_vectors(array_name);
    }
    if input
        .normals()
        .is_some_and(|array| array.name() == array_name)
    {
        output.set_active_normals(array_name);
    }
    if input
        .tcoords()
        .is_some_and(|array| array.name() == array_name)
    {
        output.set_active_tcoords(array_name);
    }
    if input
        .tensors()
        .is_some_and(|array| array.name() == array_name)
    {
        output.set_active_tensors(array_name);
    }
    if input
        .global_ids()
        .is_some_and(|array| array.name() == array_name)
    {
        output.set_active_global_ids(array_name);
    }
    if input
        .pedigree_ids()
        .is_some_and(|array| array.name() == array_name)
    {
        output.set_active_pedigree_ids(array_name);
    }
    if input
        .edge_flags()
        .is_some_and(|array| array.name() == array_name)
    {
        output.set_active_edge_flags(array_name);
    }
    if input
        .tangents()
        .is_some_and(|array| array.name() == array_name)
    {
        output.set_active_tangents(array_name);
    }
    if input
        .rational_weights()
        .is_some_and(|array| array.name() == array_name)
    {
        output.set_active_rational_weights(array_name);
    }
    if input
        .higher_order_degrees()
        .is_some_and(|array| array.name() == array_name)
    {
        output.set_active_higher_order_degrees(array_name);
    }
    if input
        .process_ids()
        .is_some_and(|array| array.name() == array_name)
    {
        output.set_active_process_ids(array_name);
    }
}

fn copy_active_attributes(input: &DataSetAttributes, output: &mut DataSetAttributes) {
    if let Some(array) = input.scalars() {
        output.set_active_scalars(array.name());
    }
    if let Some(array) = input.vectors() {
        output.set_active_vectors(array.name());
    }
    if let Some(array) = input.normals() {
        output.set_active_normals(array.name());
    }
    if let Some(array) = input.tcoords() {
        output.set_active_tcoords(array.name());
    }
    if let Some(array) = input.tensors() {
        output.set_active_tensors(array.name());
    }
    if let Some(array) = input.global_ids() {
        output.set_active_global_ids(array.name());
    }
    if let Some(array) = input.pedigree_ids() {
        output.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = input.edge_flags() {
        output.set_active_edge_flags(array.name());
    }
    if let Some(array) = input.tangents() {
        output.set_active_tangents(array.name());
    }
    if let Some(array) = input.rational_weights() {
        output.set_active_rational_weights(array.name());
    }
    if let Some(array) = input.higher_order_degrees() {
        output.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = input.process_ids() {
        output.set_active_process_ids(array.name());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_v2f() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![3.0, 6.0, 9.0],
                1,
            )));
        let r = vertex_to_face_average(&m, "s");
        let mut buf = [0.0];
        r.cell_data()
            .get_array("s")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 6.0).abs() < 1e-10);
    }
    #[test]
    fn test_f2v() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "c",
                vec![10.0, 20.0],
                1,
            )));
        let r = face_to_vertex_average(&m, "c");
        let mut buf = [0.0];
        r.point_data()
            .get_array("c")
            .unwrap()
            .tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 15.0).abs() < 1e-10);
    }
}
