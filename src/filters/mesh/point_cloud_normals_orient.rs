use crate::data::{AnyDataArray, DataArray, KdTree, PolyData};

/// Orient point cloud normals consistently using minimum spanning tree propagation.
///
/// If normals exist, propagates orientation from the point with max Z.
/// Each normal is flipped if it disagrees with its already-oriented nearest neighbor.
pub fn orient_point_cloud_normals(input: &PolyData) -> PolyData {
    let n = input.points.len();
    let arr = match input.point_data().get_array("Normals") {
        Some(a) if a.num_components() == 3 => a,
        _ => return input.clone(),
    };

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let mut normals: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            let mut buf = [0.0f64; 3];
            arr.tuple_as_f64(i, &mut buf);
            buf
        })
        .collect();

    let tree = KdTree::build(&pts);

    // Start from highest Z point
    let start = (0..n)
        .max_by(|&a, &b| pts[a][2].partial_cmp(&pts[b][2]).unwrap())
        .unwrap_or(0);
    // Ensure start normal points up
    if normals[start][2] < 0.0 {
        normals[start] = [-normals[start][0], -normals[start][1], -normals[start][2]];
    }

    // BFS propagation via k-NN
    let mut oriented = vec![false; n];
    let mut queue = std::collections::VecDeque::new();
    oriented[start] = true;
    queue.push_back(start);

    while let Some(i) = queue.pop_front() {
        let knn = tree.k_nearest(pts[i], 8);
        for &(j, _) in &knn {
            if oriented[j] {
                continue;
            }
            let dot = normals[i][0] * normals[j][0]
                + normals[i][1] * normals[j][1]
                + normals[i][2] * normals[j][2];
            if dot < 0.0 {
                normals[j] = [-normals[j][0], -normals[j][1], -normals[j][2]];
            }
            oriented[j] = true;
            queue.push_back(j);
        }
    }

    let flat: Vec<f64> = normals.iter().flat_map(|n| n.iter().copied()).collect();
    let mut pd = input.clone();
    let mut attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == "Normals" {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                flat.clone(),
                3,
            )));
        } else {
            attrs.add_array(a.clone());
        }
    }
    *pd.point_data_mut() = attrs;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orients_consistently() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 1.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        // Mixed normals: first up, second down
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 1.0],
                3,
            )));

        let result = orient_point_cloud_normals(&pd);
        let arr = result.point_data().get_array("Normals").unwrap();
        let mut buf = [0.0f64; 3];
        // All should agree after orientation
        arr.tuple_as_f64(0, &mut buf);
        let z0 = buf[2];
        arr.tuple_as_f64(1, &mut buf);
        let z1 = buf[2];
        assert!(z0 * z1 > 0.0); // same sign
    }

    #[test]
    fn no_normals_passthrough() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = orient_point_cloud_normals(&pd);
        assert_eq!(result.points.len(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = orient_point_cloud_normals(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
