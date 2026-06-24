//! Remap a scalar field from one range to another.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn scalar_remap(mesh: &PolyData, scalar_name: &str, new_min: f64, new_max: f64) -> PolyData {
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
    let range = (vmax - vmin).max(1e-15);
    let remapped: Vec<f64> = vals
        .iter()
        .map(|&v| new_min + (v - vmin) / range * (new_max - new_min))
        .collect();
    let out = format!("{}_remap", scalar_name);
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&out, remapped, 1)));
    result.point_data_mut().set_active_scalars(&out);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_remap() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 5.0, 10.0],
                1,
            )));
        let r = scalar_remap(&mesh, "v", -1.0, 1.0);
        let arr = r.point_data().get_array("v_remap").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], -1.0);
        arr.tuple_as_f64(2, &mut b);
        assert_eq!(b[0], 1.0);
    }
}
