//! Invert a scalar field: new = max + min - old.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn scalar_invert(mesh: &PolyData, scalar_name: &str) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let mut vals = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vals.push(buf[0]);
    }
    let vmin = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let vmax = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let inverted: Vec<f64> = vals.iter().map(|&v| vmax + vmin - v).collect();
    let out = format!("{}_inv", scalar_name);
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&out, inverted, 1)));
    result.point_data_mut().set_active_scalars(&out);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_invert() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        let r = scalar_invert(&mesh, "v");
        let arr = r.point_data().get_array("v_inv").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 3.0);
        arr.tuple_as_f64(2, &mut b);
        assert_eq!(b[0], 1.0);
    }
}
