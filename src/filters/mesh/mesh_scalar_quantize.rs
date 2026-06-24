//! Quantize a scalar field to N discrete levels.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn scalar_quantize(mesh: &PolyData, scalar_name: &str, n_levels: usize) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let nl = n_levels.max(2);
    let mut vals = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vals.push(buf[0]);
    }
    let vmin = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let vmax = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (vmax - vmin).max(1e-15);
    let quantized: Vec<f64> = vals
        .iter()
        .map(|&v| {
            let t = (v - vmin) / range;
            let level = (t * nl as f64).floor().min((nl - 1) as f64);
            vmin + (level + 0.5) * range / nl as f64
        })
        .collect();
    let out = format!("{}_quantized", scalar_name);
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&out, quantized, 1)));
    result.point_data_mut().set_active_scalars(&out);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_quantize() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.5, 1.0],
                1,
            )));
        let r = scalar_quantize(&mesh, "v", 4);
        let arr = r.point_data().get_array("v_quantized").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 0.125); // first quarter bin center
    }
}
