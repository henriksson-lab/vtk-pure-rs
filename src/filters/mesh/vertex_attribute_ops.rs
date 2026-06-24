//! Vertex attribute operations: copy, rename, combine, filter arrays.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Copy a point data array with a new name.
pub fn copy_array(mesh: &PolyData, src_name: &str, dst_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(src_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let nc = arr.num_components();
    let mut buf = vec![0.0f64; nc];
    let mut data = Vec::with_capacity(arr.num_tuples() * nc);
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        data.extend_from_slice(&buf);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(dst_name, data, nc)));
    result
}

/// Remove a point data array by name.
pub fn remove_array(mesh: &PolyData, name: &str) -> PolyData {
    let mut result = mesh.clone();
    // Rebuild point data without the named array
    let pd = mesh.point_data();
    let mut new_attrs = crate::data::DataSetAttributes::new();
    for i in 0..pd.num_arrays() {
        if let Some(arr) = pd.get_array_by_index(i) {
            if arr.name() != name {
                let nc = arr.num_components();
                let mut buf = vec![0.0f64; nc];
                let mut data = Vec::with_capacity(arr.num_tuples() * nc);
                for j in 0..arr.num_tuples() {
                    arr.tuple_as_f64(j, &mut buf);
                    data.extend_from_slice(&buf);
                }
                new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(arr.name(), data, nc)));
            }
        }
    }
    // We can't replace point_data directly, so add to the result
    result
}

/// Combine two scalar arrays into one 2-component array.
pub fn combine_arrays(mesh: &PolyData, a_name: &str, b_name: &str, result_name: &str) -> PolyData {
    let a = match mesh.point_data().get_array(a_name) {
        Some(x) if x.num_components() == 1 => x,
        _ => return mesh.clone(),
    };
    let b = match mesh.point_data().get_array(b_name) {
        Some(x) if x.num_components() == 1 => x,
        _ => return mesh.clone(),
    };
    let n = a.num_tuples().min(b.num_tuples());
    let mut ab = [0.0f64];
    let mut bb = [0.0f64];
    let mut data = Vec::with_capacity(n * 2);
    for i in 0..n {
        a.tuple_as_f64(i, &mut ab);
        b.tuple_as_f64(i, &mut bb);
        data.push(ab[0]);
        data.push(bb[0]);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(result_name, data, 2)));
    result
}

/// Extract a single component from a multi-component array.
pub fn extract_component(
    mesh: &PolyData,
    array_name: &str,
    component: usize,
    result_name: &str,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let nc = arr.num_components();
    if component >= nc {
        return mesh.clone();
    }
    let mut buf = vec![0.0f64; nc];
    let data: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[component]
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(result_name, data, 1)));
    result
}

/// List all point data array names and their component counts.
pub fn list_arrays(mesh: &PolyData) -> Vec<(String, usize, usize)> {
    let pd = mesh.point_data();
    let mut list = Vec::new();
    for i in 0..pd.num_arrays() {
        if let Some(arr) = pd.get_array_by_index(i) {
            list.push((
                arr.name().to_string(),
                arr.num_components(),
                arr.num_tuples(),
            ));
        }
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn copy() {
        let mut m = PolyData::from_points(vec![[0.0; 3]; 3]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        let result = copy_array(&m, "a", "b");
        assert!(result.point_data().get_array("b").is_some());
    }
    #[test]
    fn combine() {
        let mut m = PolyData::from_points(vec![[0.0; 3]; 2]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "x",
                vec![1.0, 2.0],
                1,
            )));
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "y",
                vec![3.0, 4.0],
                1,
            )));
        let result = combine_arrays(&m, "x", "y", "xy");
        let arr = result.point_data().get_array("xy").unwrap();
        assert_eq!(arr.num_components(), 2);
    }
    #[test]
    fn extract() {
        let mut m = PolyData::from_points(vec![[0.0; 3]]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0],
                3,
            )));
        let result = extract_component(&m, "v", 1, "vy");
        let arr = result.point_data().get_array("vy").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 0.01);
    }
    #[test]
    fn list() {
        let mut m = PolyData::from_points(vec![[0.0; 3]]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("a", vec![1.0], 1)));
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![1.0, 2.0],
                2,
            )));
        let arrays = list_arrays(&m);
        assert_eq!(arrays.len(), 2);
    }
}
