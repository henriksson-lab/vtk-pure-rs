//! Clamp a scalar field to a given range.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn scalar_clamp(mesh: &PolyData, scalar_name: &str, min_val: f64, max_val: f64) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let mut vals = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vals.push(buf[0].clamp(min_val, max_val));
    }
    let out_name = format!("{}_clamped", scalar_name);
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&out_name, vals, 1)));
    result.point_data_mut().set_active_scalars(&out_name);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_clamp() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![-5.0, 0.5, 10.0],
                1,
            )));
        let r = scalar_clamp(&mesh, "v", 0.0, 1.0);
        let arr = r.point_data().get_array("v_clamped").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 0.0);
        arr.tuple_as_f64(2, &mut b);
        assert_eq!(b[0], 1.0);
    }
}
