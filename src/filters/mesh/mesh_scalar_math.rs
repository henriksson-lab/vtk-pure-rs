//! Pointwise scalar math: add, multiply, power, log, abs on scalar fields.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn scalar_add(mesh: &PolyData, scalar_name: &str, value: f64) -> PolyData {
    apply_op(mesh, scalar_name, &format!("{}_add", scalar_name), |v| {
        v + value
    })
}

pub fn scalar_multiply(mesh: &PolyData, scalar_name: &str, factor: f64) -> PolyData {
    apply_op(mesh, scalar_name, &format!("{}_mul", scalar_name), |v| {
        v * factor
    })
}

pub fn scalar_power(mesh: &PolyData, scalar_name: &str, exponent: f64) -> PolyData {
    apply_op(mesh, scalar_name, &format!("{}_pow", scalar_name), |v| {
        v.abs().max(1e-30).powf(exponent)
    })
}

pub fn scalar_log(mesh: &PolyData, scalar_name: &str) -> PolyData {
    apply_op(mesh, scalar_name, &format!("{}_log", scalar_name), |v| {
        v.abs().max(1e-30).ln()
    })
}

pub fn scalar_abs(mesh: &PolyData, scalar_name: &str) -> PolyData {
    apply_op(mesh, scalar_name, &format!("{}_abs", scalar_name), |v| {
        v.abs()
    })
}

fn apply_op(
    mesh: &PolyData,
    scalar_name: &str,
    out_name: &str,
    op: impl Fn(f64) -> f64,
) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let mut vals = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vals.push(op(buf[0]));
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
    fn test_math() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![2.0, 4.0, 6.0],
                1,
            )));
        let r = scalar_multiply(&mesh, "v", 3.0);
        let arr = r.point_data().get_array("v_mul").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 6.0);
    }
}
