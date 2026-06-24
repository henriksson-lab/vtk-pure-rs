use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the centroid of each polygon face and store as cell data.
///
/// Adds a 3-component "FaceCentroid" array to cell data, where each tuple
/// is the average of the vertex positions of the corresponding face.
pub fn compute_face_centroids(input: &PolyData) -> PolyData {
    let mut centroids: Vec<f64> = Vec::new();

    for cell in input.polys.iter() {
        if cell.is_empty() {
            centroids.extend_from_slice(&[0.0, 0.0, 0.0]);
            continue;
        }

        let n: f64 = cell.len() as f64;
        let mut cx: f64 = 0.0;
        let mut cy: f64 = 0.0;
        let mut cz: f64 = 0.0;

        for &idx in cell.iter() {
            let p = input.points.get(idx as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }

        centroids.push(cx / n);
        centroids.push(cy / n);
        centroids.push(cz / n);
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FaceCentroid",
            centroids,
            3,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_centroid() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_face_centroids(&pd);
        let arr = result.cell_data().get_array("FaceCentroid").unwrap();
        assert_eq!(arr.num_tuples(), 1);
        assert_eq!(arr.num_components(), 3);
        let mut val = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 1.0).abs() < 1e-10);
        assert!((val[1] - 1.0).abs() < 1e-10);
        assert!((val[2] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn two_triangles_centroids() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [6.0, 0.0, 0.0],
                [0.0, 6.0, 0.0],
                [6.0, 6.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = compute_face_centroids(&pd);
        let arr = result.cell_data().get_array("FaceCentroid").unwrap();
        assert_eq!(arr.num_tuples(), 2);
        let mut val = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 2.0).abs() < 1e-10);
        assert!((val[1] - 2.0).abs() < 1e-10);
        arr.tuple_as_f64(1, &mut val);
        assert!((val[0] - 4.0).abs() < 1e-10);
        assert!((val[1] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn quad_centroid() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([4.0, 0.0, 0.0]);
        pd.points.push([4.0, 4.0, 0.0]);
        pd.points.push([0.0, 4.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);
        let result = compute_face_centroids(&pd);
        let arr = result.cell_data().get_array("FaceCentroid").unwrap();
        let mut val = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 2.0).abs() < 1e-10);
        assert!((val[1] - 2.0).abs() < 1e-10);
        assert!((val[2] - 0.0).abs() < 1e-10);
    }
}
