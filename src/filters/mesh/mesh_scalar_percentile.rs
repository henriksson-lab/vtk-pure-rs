//! Compute percentile rank of each vertex's scalar value.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn scalar_percentile(mesh: &PolyData, scalar_name: &str) -> PolyData {
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
    let mut sorted = vals.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let percentiles: Vec<f64> = vals
        .iter()
        .map(|&v| {
            let rank = sorted.partition_point(|&x| x < v);
            rank as f64 / n.max(1) as f64 * 100.0
        })
        .collect();
    let out = format!("{}_percentile", scalar_name);
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&out, percentiles, 1)));
    result.point_data_mut().set_active_scalars(&out);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_percentile() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0, 20.0, 30.0],
                1,
            )));
        let r = scalar_percentile(&mesh, "v");
        let arr = r.point_data().get_array("v_percentile").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 0.0); // lowest
        arr.tuple_as_f64(2, &mut b);
        assert!((b[0] - 66.67).abs() < 1.0); // highest
    }
}
