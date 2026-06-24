//! Combine two scalar fields: add, subtract, multiply, min, max.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn scalar_add_fields(mesh: &PolyData, name_a: &str, name_b: &str, out_name: &str) -> PolyData {
    combine(mesh, name_a, name_b, out_name, |a, b| a + b)
}

pub fn scalar_subtract_fields(
    mesh: &PolyData,
    name_a: &str,
    name_b: &str,
    out_name: &str,
) -> PolyData {
    combine(mesh, name_a, name_b, out_name, |a, b| a - b)
}

pub fn scalar_multiply_fields(
    mesh: &PolyData,
    name_a: &str,
    name_b: &str,
    out_name: &str,
) -> PolyData {
    combine(mesh, name_a, name_b, out_name, |a, b| a * b)
}

pub fn scalar_min_fields(mesh: &PolyData, name_a: &str, name_b: &str, out_name: &str) -> PolyData {
    combine(mesh, name_a, name_b, out_name, |a, b| a.min(b))
}

pub fn scalar_max_fields(mesh: &PolyData, name_a: &str, name_b: &str, out_name: &str) -> PolyData {
    combine(mesh, name_a, name_b, out_name, |a, b| a.max(b))
}

fn combine(
    mesh: &PolyData,
    name_a: &str,
    name_b: &str,
    out_name: &str,
    op: impl Fn(f64, f64) -> f64,
) -> PolyData {
    let n = mesh.points.len();
    let arr_a = match mesh.point_data().get_array(name_a) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let arr_b = match mesh.point_data().get_array(name_b) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let mut vals = Vec::with_capacity(n);
    let mut ba = [0.0f64];
    let mut bb = [0.0f64];
    for i in 0..n {
        arr_a.tuple_as_f64(i, &mut ba);
        arr_b.tuple_as_f64(i, &mut bb);
        vals.push(op(ba[0], bb[0]));
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(out_name, vals, 1)));
    result.point_data_mut().set_active_scalars(out_name);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_combine() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![10.0, 20.0, 30.0],
                1,
            )));
        let r = scalar_add_fields(&mesh, "a", "b", "sum");
        let arr = r.point_data().get_array("sum").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 11.0);
    }
}
