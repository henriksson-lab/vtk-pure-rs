//! Array operations on mesh point/cell data.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Scale all values in a point data array by a factor.
pub fn scale_array(mesh: &PolyData, array_name: &str, factor: f64) -> PolyData {
    map_array(mesh, array_name, |v| v * factor)
}

/// Offset all values in a point data array.
pub fn offset_array(mesh: &PolyData, array_name: &str, offset: f64) -> PolyData {
    map_array(mesh, array_name, |v| v + offset)
}

/// Clamp array values to [min, max].
pub fn clamp_array(mesh: &PolyData, array_name: &str, min: f64, max: f64) -> PolyData {
    map_array(mesh, array_name, |v| v.clamp(min, max))
}

/// Normalize array to [0, 1].
pub fn normalize_array(mesh: &PolyData, array_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (mx - mn).abs() < 1e-15 {
        1.0
    } else {
        mx - mn
    };
    let data: Vec<f64> = vals.iter().map(|&v| (v - mn) / range).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

/// Compute magnitude of a vector array.
pub fn vector_magnitude(mesh: &PolyData, array_name: &str, output_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() >= 2 => a,
        _ => return mesh.clone(),
    };
    let nc = arr.num_components();
    let n = arr.num_tuples();
    let mut buf = vec![0.0f64; nc];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf.iter().map(|v| v * v).sum::<f64>().sqrt()
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(output_name, data, 1)));
    result
}

fn map_array(mesh: &PolyData, array_name: &str, f: impl Fn(f64) -> f64) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            f(buf[0])
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_scale() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        let r = scale_array(&mesh, "s", 10.0);
        let arr = r.point_data().get_array("s").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 20.0).abs() < 1e-10);
    }
    #[test]
    fn test_normalize() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![10.0, 20.0, 30.0],
                1,
            )));
        let r = normalize_array(&mesh, "s");
        let arr = r.point_data().get_array("s").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }
    #[test]
    fn test_magnitude() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                3,
            )));
        let r = vector_magnitude(&mesh, "v", "mag");
        let arr = r.point_data().get_array("mag").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }
}
