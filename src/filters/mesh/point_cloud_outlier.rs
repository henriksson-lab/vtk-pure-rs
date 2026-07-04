use crate::data::{
    AnyDataArray, CellArray, DataArray, DataSetAttributes, KdTree, Points, PolyData,
};

/// Detect outliers in a point cloud using statistical analysis.
///
/// For each point, computes mean distance to k nearest neighbors.
/// Points where the mean distance exceeds `mean + n_sigma * stddev`
/// are marked as outliers. Adds "IsOutlier" (0/1) and "MeanKnnDist".
pub fn detect_outliers(input: &PolyData, k: usize, n_sigma: f64) -> PolyData {
    let n = input.points.len();
    if n < 2 {
        return input.clone();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);
    let k = k.max(1).min(n - 1);

    let mut mean_dists = vec![0.0f64; n];
    for i in 0..n {
        let knn = tree.k_nearest(pts[i], k + 1);
        let sum: f64 = knn.iter().skip(1).map(|&(_, d2)| d2.sqrt()).sum();
        mean_dists[i] = sum / k as f64;
    }

    let global_mean: f64 = mean_dists.iter().sum::<f64>() / n as f64;
    let global_var: f64 = mean_dists
        .iter()
        .map(|d| (d - global_mean).powi(2))
        .sum::<f64>()
        / n as f64;
    let global_std = global_var.sqrt();
    let threshold = global_mean + n_sigma * global_std;

    let outliers: Vec<f64> = mean_dists
        .iter()
        .map(|&d| if d > threshold { 1.0 } else { 0.0 })
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MeanKnnDist",
            mean_dists,
            1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "IsOutlier",
            outliers,
            1,
        )));
    pd
}

/// Remove outlier points from a point cloud.
pub fn remove_outliers(input: &PolyData, k: usize, n_sigma: f64) -> PolyData {
    let detected = detect_outliers(input, k, n_sigma);
    let arr = match detected.point_data().get_array("IsOutlier") {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = input.points.len();
    let mut out_pts = Points::<f64>::new();
    let mut out_verts = CellArray::new();
    let mut kept_point_ids = Vec::new();
    let mut buf = [0.0f64];

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] < 0.5 {
            let idx = out_pts.len() as i64;
            out_pts.push(input.points.get(i));
            out_verts.push_cell(&[idx]);
            kept_point_ids.push(i);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.verts = out_verts;
    *pd.point_data_mut() = subset_point_data(input.point_data(), &kept_point_ids, n);
    pd
}

fn subset_point_data(
    input: &DataSetAttributes,
    point_ids: &[usize],
    number_of_input_points: usize,
) -> DataSetAttributes {
    let mut output = DataSetAttributes::new();

    for array in input.iter() {
        if array.num_tuples() == number_of_input_points {
            output.add_array(subset_array_tuples(array, point_ids));
        }
    }

    if let Some(array) = input.scalars() {
        output.set_active_scalars(array.name());
    }
    if let Some(array) = input.vectors() {
        output.set_active_vectors(array.name());
    }
    if let Some(array) = input.normals() {
        output.set_active_normals(array.name());
    }
    if let Some(array) = input.tcoords() {
        output.set_active_tcoords(array.name());
    }
    if let Some(array) = input.tensors() {
        output.set_active_tensors(array.name());
    }
    if let Some(array) = input.global_ids() {
        output.set_active_global_ids(array.name());
    }
    if let Some(array) = input.pedigree_ids() {
        output.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = input.edge_flags() {
        output.set_active_edge_flags(array.name());
    }
    if let Some(array) = input.tangents() {
        output.set_active_tangents(array.name());
    }
    if let Some(array) = input.rational_weights() {
        output.set_active_rational_weights(array.name());
    }
    if let Some(array) = input.higher_order_degrees() {
        output.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = input.process_ids() {
        output.set_active_process_ids(array.name());
    }

    output
}

fn subset_array_tuples(array: &AnyDataArray, point_ids: &[usize]) -> AnyDataArray {
    macro_rules! subset {
        ($array:expr, $variant:ident) => {{
            let number_of_components = $array.num_components();
            let mut values = Vec::with_capacity(point_ids.len() * number_of_components);
            for &point_id in point_ids {
                values.extend_from_slice($array.tuple(point_id));
            }
            AnyDataArray::$variant(DataArray::from_vec(
                $array.name(),
                values,
                number_of_components,
            ))
        }};
    }

    match array {
        AnyDataArray::F32(a) => subset!(a, F32),
        AnyDataArray::F64(a) => subset!(a, F64),
        AnyDataArray::I8(a) => subset!(a, I8),
        AnyDataArray::I16(a) => subset!(a, I16),
        AnyDataArray::I32(a) => subset!(a, I32),
        AnyDataArray::I64(a) => subset!(a, I64),
        AnyDataArray::U8(a) => subset!(a, U8),
        AnyDataArray::U16(a) => subset!(a, U16),
        AnyDataArray::U32(a) => subset!(a, U32),
        AnyDataArray::U64(a) => subset!(a, U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_far_point() {
        let mut pd = PolyData::new();
        for i in 0..20 {
            pd.points.push([i as f64 * 0.1, 0.0, 0.0]);
        }
        pd.points.push([100.0, 0.0, 0.0]); // outlier

        let result = detect_outliers(&pd, 3, 2.0);
        let arr = result.point_data().get_array("IsOutlier").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(20, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(10, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn remove_outlier() {
        let mut pd = PolyData::new();
        for i in 0..10 {
            pd.points.push([i as f64 * 0.1, 0.0, 0.0]);
        }
        pd.points.push([100.0, 0.0, 0.0]);

        let result = remove_outliers(&pd, 3, 2.0);
        assert!(result.points.len() < 11);
        assert!(result.points.len() >= 10);
    }

    #[test]
    fn uniform_no_outliers() {
        let mut pd = PolyData::new();
        for i in 0..10 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }

        let result = detect_outliers(&pd, 3, 3.0);
        let arr = result.point_data().get_array("IsOutlier").unwrap();
        let mut buf = [0.0f64];
        let mut outlier_count = 0;
        for i in 0..10 {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                outlier_count += 1;
            }
        }
        assert!(outlier_count <= 2); // maybe endpoints
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = detect_outliers(&pd, 3, 2.0);
        assert_eq!(result.points.len(), 0);
    }
}
