//! Thicken a surface mesh into a solid shell.
use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use crate::types::Scalar;

pub fn thicken(mesh: &PolyData, thickness: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let normals = compute_normals(mesh);
    let half = thickness / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        let nm = &normals[i];
        pts.push([
            p[0] + nm[0] * half,
            p[1] + nm[1] * half,
            p[2] + nm[2] * half,
        ]);
    }
    for i in 0..n {
        let p = mesh.points.get(i);
        let nm = &normals[i];
        pts.push([
            p[0] - nm[0] * half,
            p[1] - nm[1] * half,
            p[2] - nm[2] * half,
        ]);
    }
    for cell in mesh.polys.iter() {
        if !valid_cell(cell, n) {
            continue;
        }
        polys.push_cell(cell);
        let mut top: Vec<i64> = cell.iter().map(|&v| v + n as i64).collect();
        top.reverse();
        polys.push_cell(&top);
    }
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        if !valid_cell(cell, n) {
            continue;
        }
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
    }
    for (&(a, b), &c) in &ec {
        if c == 1 {
            polys.push_cell(&[a as i64, b as i64, (b + n) as i64, (a + n) as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    *r.field_data_mut() = mesh.field_data().clone();
    duplicate_point_data(mesh, &mut r);
    r
}
fn compute_normals(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 || !valid_cell(cell, n) {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &v in cell {
            let vi = v as usize;
            if vi < n {
                nm[vi][0] += fn_[0];
                nm[vi][1] += fn_[1];
                nm[vi][2] += fn_[2];
            }
        }
    }
    for v in &mut nm {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 1e-15 {
            v[0] /= l;
            v[1] /= l;
            v[2] /= l;
        }
    }
    nm
}

fn valid_cell(cell: &[i64], number_of_points: usize) -> bool {
    cell.iter()
        .all(|&id| id >= 0 && (id as usize) < number_of_points)
}

fn duplicate_point_data(input: &PolyData, output: &mut PolyData) {
    let n = input.points.len();
    let input_point_data = input.point_data();

    let active_scalars = input_point_data
        .scalars()
        .map(|array| array.name().to_string());
    let active_vectors = input_point_data
        .vectors()
        .map(|array| array.name().to_string());
    let active_normals = input_point_data
        .normals()
        .map(|array| array.name().to_string());
    let active_tcoords = input_point_data
        .tcoords()
        .map(|array| array.name().to_string());
    let active_tensors = input_point_data
        .tensors()
        .map(|array| array.name().to_string());
    let active_global_ids = input_point_data
        .global_ids()
        .map(|array| array.name().to_string());
    let active_pedigree_ids = input_point_data
        .pedigree_ids()
        .map(|array| array.name().to_string());
    let active_edge_flags = input_point_data
        .edge_flags()
        .map(|array| array.name().to_string());
    let active_tangents = input_point_data
        .tangents()
        .map(|array| array.name().to_string());
    let active_rational_weights = input_point_data
        .rational_weights()
        .map(|array| array.name().to_string());
    let active_higher_order_degrees = input_point_data
        .higher_order_degrees()
        .map(|array| array.name().to_string());
    let active_process_ids = input_point_data
        .process_ids()
        .map(|array| array.name().to_string());

    for ai in 0..input_point_data.num_arrays() {
        if let Some(array) = input_point_data.get_array_by_index(ai) {
            if array.num_tuples() != n {
                continue;
            }
            let point_ids: Vec<usize> = (0..n).chain(0..n).collect();
            output
                .point_data_mut()
                .add_array(copy_array_tuples(array, &point_ids));
        }
    }

    let output_point_data = output.point_data_mut();
    if let Some(name) = active_scalars {
        output_point_data.set_active_scalars(&name);
    }
    if let Some(name) = active_vectors {
        output_point_data.set_active_vectors(&name);
    }
    if let Some(name) = active_normals {
        output_point_data.set_active_normals(&name);
    }
    if let Some(name) = active_tcoords {
        output_point_data.set_active_tcoords(&name);
    }
    if let Some(name) = active_tensors {
        output_point_data.set_active_tensors(&name);
    }
    if let Some(name) = active_global_ids {
        output_point_data.set_active_global_ids(&name);
    }
    if let Some(name) = active_pedigree_ids {
        output_point_data.set_active_pedigree_ids(&name);
    }
    if let Some(name) = active_edge_flags {
        output_point_data.set_active_edge_flags(&name);
    }
    if let Some(name) = active_tangents {
        output_point_data.set_active_tangents(&name);
    }
    if let Some(name) = active_rational_weights {
        output_point_data.set_active_rational_weights(&name);
    }
    if let Some(name) = active_higher_order_degrees {
        output_point_data.set_active_higher_order_degrees(&name);
    }
    if let Some(name) = active_process_ids {
        output_point_data.set_active_process_ids(&name);
    }
}

fn copy_array_tuples(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! copy_typed {
        ($arr:expr, $variant:path) => {{
            $variant(copy_typed_array($arr, tuple_ids))
        }};
    }

    match array {
        AnyDataArray::F32(a) => copy_typed!(a, AnyDataArray::F32),
        AnyDataArray::F64(a) => copy_typed!(a, AnyDataArray::F64),
        AnyDataArray::I8(a) => copy_typed!(a, AnyDataArray::I8),
        AnyDataArray::I16(a) => copy_typed!(a, AnyDataArray::I16),
        AnyDataArray::I32(a) => copy_typed!(a, AnyDataArray::I32),
        AnyDataArray::I64(a) => copy_typed!(a, AnyDataArray::I64),
        AnyDataArray::U8(a) => copy_typed!(a, AnyDataArray::U8),
        AnyDataArray::U16(a) => copy_typed!(a, AnyDataArray::U16),
        AnyDataArray::U32(a) => copy_typed!(a, AnyDataArray::U32),
        AnyDataArray::U64(a) => copy_typed!(a, AnyDataArray::U64),
    }
}

fn copy_typed_array<T: Scalar>(array: &DataArray<T>, tuple_ids: &[usize]) -> DataArray<T> {
    let mut output = DataArray::new(array.name(), array.num_components());
    for &tuple_id in tuple_ids {
        output.push_tuple(array.tuple(tuple_id));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_thicken() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = thicken(&m, 0.5);
        assert_eq!(r.points.len(), 6);
        assert!(r.polys.num_cells() > 2);
    }

    #[test]
    fn test_thicken_duplicates_point_data() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "ids",
                vec![3, 5, 7],
                1,
            )));
        let r = thicken(&m, 0.5);
        match r.point_data().get_array("ids").unwrap() {
            AnyDataArray::I32(array) => {
                assert_eq!(array.num_tuples(), 6);
                assert_eq!(array.tuple(0), &[3]);
                assert_eq!(array.tuple(3), &[3]);
            }
            other => panic!("unexpected array type: {:?}", other.scalar_type()),
        }
    }
}
