//! Mirror mesh across planes.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, PolyData};

/// Mirror mesh across the YZ plane (flip X).
pub fn mirror_x(mesh: &PolyData) -> PolyData {
    mirror_axis(mesh, 0)
}

/// Mirror mesh across the XZ plane (flip Y).
pub fn mirror_y(mesh: &PolyData) -> PolyData {
    mirror_axis(mesh, 1)
}

/// Mirror mesh across the XY plane (flip Z).
pub fn mirror_z(mesh: &PolyData) -> PolyData {
    mirror_axis(mesh, 2)
}

/// Mirror mesh across an arbitrary plane defined by point and normal.
pub fn mirror_plane(mesh: &PolyData, point: [f64; 3], normal: [f64; 3]) -> PolyData {
    let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if len < 1e-15 {
        return mesh.clone();
    }
    let n = [normal[0] / len, normal[1] / len, normal[2] / len];
    let mut result = mesh.clone();
    for i in 0..result.points.len() {
        let p = result.points.get(i);
        let d = (p[0] - point[0]) * n[0] + (p[1] - point[1]) * n[1] + (p[2] - point[2]) * n[2];
        result.points.set(
            i,
            [
                p[0] - 2.0 * d * n[0],
                p[1] - 2.0 * d * n[1],
                p[2] - 2.0 * d * n[2],
            ],
        );
    }
    // Reverse surface-cell winding to maintain outward normals.
    result.polys = reversed_cells(&result.polys);
    result.strips = reflected_strips(&result.strips);
    reflect_active_attributes(&mut result, Mirror::Plane(n));
    result
}

fn mirror_axis(mesh: &PolyData, axis: usize) -> PolyData {
    let mut result = mesh.clone();
    for i in 0..result.points.len() {
        let mut p = result.points.get(i);
        p[axis] = -p[axis];
        result.points.set(i, p);
    }
    result.polys = reversed_cells(&result.polys);
    result.strips = reflected_strips(&result.strips);
    reflect_active_attributes(&mut result, Mirror::Axis(axis));
    result
}

fn reversed_cells(cells: &CellArray) -> CellArray {
    let mut reversed_cells = CellArray::new();
    for cell in cells.iter() {
        let mut reversed: Vec<i64> = cell.to_vec();
        reversed.reverse();
        reversed_cells.push_cell(&reversed);
    }
    reversed_cells
}

fn reflected_strips(strips: &CellArray) -> CellArray {
    let mut reflected = CellArray::new();
    for strip in strips.iter() {
        if strip.len() >= 4 && strip.len() % 2 == 0 {
            let mut cell = Vec::with_capacity(strip.len() + 1);
            cell.extend_from_slice(&[strip[0], strip[2], strip[1], strip[2]]);
            cell.extend_from_slice(&strip[3..]);
            reflected.push_cell(&cell);
        } else {
            let mut reversed: Vec<i64> = strip.to_vec();
            reversed.reverse();
            reflected.push_cell(&reversed);
        }
    }
    reflected
}

#[derive(Clone, Copy)]
enum Mirror {
    Axis(usize),
    Plane([f64; 3]),
}

fn reflect_active_attributes(mesh: &mut PolyData, mirror: Mirror) {
    reflect_active_attributes_for(mesh.point_data_mut(), mirror);
    reflect_active_attributes_for(mesh.cell_data_mut(), mirror);
}

fn reflect_active_attributes_for(attrs: &mut DataSetAttributes, mirror: Mirror) {
    let mut names = Vec::new();
    if let Some(array) = attrs.vectors() {
        names.push(array.name().to_string());
    }
    if let Some(array) = attrs.normals() {
        names.push(array.name().to_string());
    }
    if matches!(mirror, Mirror::Axis(_)) {
        if let Some(array) = attrs.tensors() {
            names.push(array.name().to_string());
        }
    }
    names.sort();
    names.dedup();

    for name in names {
        if let Some(array) = attrs.field_data_mut().get_array_mut(&name) {
            reflect_array(array, mirror);
        }
    }
}

fn reflect_array(array: &mut AnyDataArray, mirror: Mirror) {
    macro_rules! reflect {
        ($array:expr) => {
            reflect_data_array($array, mirror)
        };
    }

    match array {
        AnyDataArray::F32(array) => reflect!(array),
        AnyDataArray::F64(array) => reflect!(array),
        AnyDataArray::I8(array) => reflect!(array),
        AnyDataArray::I16(array) => reflect!(array),
        AnyDataArray::I32(array) => reflect!(array),
        AnyDataArray::I64(array) => reflect!(array),
        AnyDataArray::U8(_)
        | AnyDataArray::U16(_)
        | AnyDataArray::U32(_)
        | AnyDataArray::U64(_) => {}
    }
}

fn reflect_data_array<T>(array: &mut DataArray<T>, mirror: Mirror)
where
    T: crate::types::Scalar,
{
    match (mirror, array.num_components()) {
        (Mirror::Axis(axis), 3) => {
            for tuple in 0..array.num_tuples() {
                let t = array.tuple_mut(tuple);
                t[axis] = T::from_f64(-t[axis].to_f64());
            }
        }
        (Mirror::Axis(axis), 6) => {
            let signs = symmetric_tensor_signs(axis);
            for tuple in 0..array.num_tuples() {
                let t = array.tuple_mut(tuple);
                for i in 0..6 {
                    if signs[i] < 0 {
                        t[i] = T::from_f64(-t[i].to_f64());
                    }
                }
            }
        }
        (Mirror::Axis(axis), 9) => {
            let signs = tensor_signs(axis);
            for tuple in 0..array.num_tuples() {
                let t = array.tuple_mut(tuple);
                for i in 0..9 {
                    if signs[i] < 0 {
                        t[i] = T::from_f64(-t[i].to_f64());
                    }
                }
            }
        }
        (Mirror::Plane(normal), 3) => {
            for tuple in 0..array.num_tuples() {
                let t = array.tuple_mut(tuple);
                let v = [t[0].to_f64(), t[1].to_f64(), t[2].to_f64()];
                let d = v[0] * normal[0] + v[1] * normal[1] + v[2] * normal[2];
                t[0] = T::from_f64(v[0] - 2.0 * d * normal[0]);
                t[1] = T::from_f64(v[1] - 2.0 * d * normal[1]);
                t[2] = T::from_f64(v[2] - 2.0 * d * normal[2]);
            }
        }
        _ => {}
    }
}

fn symmetric_tensor_signs(axis: usize) -> [i8; 6] {
    let m = mirror_dir(axis);
    [
        m[0] * m[0],
        m[1] * m[1],
        m[2] * m[2],
        m[0] * m[1],
        m[1] * m[2],
        m[0] * m[2],
    ]
}

fn tensor_signs(axis: usize) -> [i8; 9] {
    let m = mirror_dir(axis);
    [
        m[0] * m[0],
        m[0] * m[1],
        m[0] * m[2],
        m[1] * m[0],
        m[1] * m[1],
        m[1] * m[2],
        m[2] * m[0],
        m[2] * m[1],
        m[2] * m[2],
    ]
}

fn mirror_dir(axis: usize) -> [i8; 3] {
    let mut m = [1, 1, 1];
    m[axis] = -1;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mirror_x() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = mirror_x(&mesh);
        let p = r.points.get(0);
        assert!((p[0] + 1.0).abs() < 1e-10);
    }
    #[test]
    fn test_mirror_plane() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [2.0, 2.0, 3.0], [1.5, 3.0, 3.0]],
            vec![[0, 1, 2]],
        );
        let r = mirror_plane(&mesh, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let p = r.points.get(0);
        assert!((p[0] + 1.0).abs() < 1e-10);
        assert!((p[1] - 2.0).abs() < 1e-10);
    }
    #[test]
    fn test_winding_reversed() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = mirror_z(&mesh);
        let cell: Vec<i64> = r.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell, vec![2, 1, 0]); // reversed
    }
    #[test]
    fn test_strip_winding_reversed() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.strips.push_cell(&[0, 1, 2, 3]);

        let r = mirror_z(&mesh);
        let cell: Vec<i64> = r.strips.iter().next().unwrap().to_vec();
        assert_eq!(cell, vec![0, 2, 1, 2, 3]);
    }

    #[test]
    fn mirror_reflects_active_vectors_and_normals() {
        use crate::data::{AnyDataArray, DataArray};

        let mut mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "vectors",
                vec![1.0, 2.0, 3.0, -4.0, 5.0, 6.0, 7.0, 8.0, 9.0],
                3,
            )));
        mesh.point_data_mut().set_active_vectors("vectors");

        let r = mirror_x(&mesh);
        assert_eq!(
            r.point_data().vectors().unwrap().to_f64_vec_flat(),
            vec![-1.0, 2.0, 3.0, 4.0, 5.0, 6.0, -7.0, 8.0, 9.0]
        );
    }
}
