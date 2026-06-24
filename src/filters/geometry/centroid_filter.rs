use crate::data::{CellArray, Points, PolyData};

/// Replace each cell with a single point at its centroid.
///
/// Creates a point cloud where each point is the centroid of a cell
/// from the input. Cell data arrays are copied as point data on the output.
pub fn centroid_filter(input: &PolyData) -> PolyData {
    let mut out_points = Points::<f64>::new();
    let mut out_verts = CellArray::new();

    for cell in input.polys.iter() {
        if cell.is_empty() {
            continue;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &id in cell.iter() {
            let p = input.points.get(id as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = cell.len() as f64;
        let idx = out_points.len() as i64;
        out_points.push([cx / n, cy / n, cz / n]);
        out_verts.push_cell(&[idx]);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;

    // Copy cell data as point data
    for i in 0..input.cell_data().num_arrays() {
        let arr = input.cell_data().get_array_by_index(i).unwrap();
        pd.point_data_mut().add_array(arr.clone());
    }
    pd
}

/// Compute the weighted centroid (center of mass) of a point cloud.
///
/// If `weight_array` is provided, uses those values as weights.
/// Otherwise uses uniform weighting. Returns the centroid position.
pub fn weighted_centroid(input: &PolyData, weight_array: Option<&str>) -> [f64; 3] {
    let n = input.points.len();
    if n == 0 {
        return [0.0, 0.0, 0.0];
    }

    let weights: Vec<f64> = if let Some(name) = weight_array {
        if let Some(arr) = input.point_data().get_array(name) {
            let mut buf = [0.0f64];
            (0..n)
                .map(|i| {
                    arr.tuple_as_f64(i, &mut buf);
                    buf[0]
                })
                .collect()
        } else {
            vec![1.0; n]
        }
    } else {
        vec![1.0; n]
    };

    let mut sum = [0.0f64; 3];
    let mut total_w = 0.0;
    for i in 0..n {
        let p = input.points.get(i);
        let w = weights[i];
        sum[0] += p[0] * w;
        sum[1] += p[1] * w;
        sum[2] += p[2] * w;
        total_w += w;
    }

    if total_w > 1e-15 {
        [sum[0] / total_w, sum[1] / total_w, sum[2] / total_w]
    } else {
        [0.0, 0.0, 0.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn cell_centroids() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([3.0, 0.0, 0.0]);
        pd.points.push([0.0, 3.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = centroid_filter(&pd);
        assert_eq!(result.points.len(), 1);
        let c = result.points.get(0);
        assert!((c[0] - 1.0).abs() < 1e-10);
        assert!((c[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn uniform_centroid() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 2.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);

        let c = weighted_centroid(&pd, None);
        assert!((c[0] - 1.0).abs() < 1e-10);
        assert!((c[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn weighted() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "w",
                vec![9.0, 1.0],
                1,
            )));

        let c = weighted_centroid(&pd, Some("w"));
        assert!((c[0] - 1.0).abs() < 1e-10); // weighted toward 0
    }

    #[test]
    fn empty_centroid() {
        let pd = PolyData::new();
        let c = weighted_centroid(&pd, None);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }
}
