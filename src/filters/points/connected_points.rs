use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::VecDeque;

/// Connected components for point clouds using a VTK-style radius neighborhood.
///
/// Two points are connected if their Euclidean distance is within `radius`.
/// A wave traversal assigns each point a "RegionLabels" value, matching
/// vtkConnectedPointsFilter's all-regions labeling mode.
///
/// Returns a copy of the input with a "RegionLabels" point data array.
pub fn connected_points(input: &PolyData, radius: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return PolyData::new();
    }

    let radius2 = radius * radius;
    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    let mut region_labels = vec![-1i64; n];
    let mut current_region_number = 0i64;

    for start in 0..n {
        if region_labels[start] >= 0 {
            continue;
        }

        let mut queue = VecDeque::new();
        queue.push_back(start);
        region_labels[start] = current_region_number;

        while let Some(idx) = queue.pop_front() {
            let p = pts[idx];
            for j in 0..n {
                if region_labels[j] >= 0 {
                    continue;
                }
                let q = pts[j];
                let dx = p[0] - q[0];
                let dy = p[1] - q[1];
                let dz = p[2] - q[2];
                if dx * dx + dy * dy + dz * dz <= radius2 {
                    region_labels[j] = current_region_number;
                    queue.push_back(j);
                }
            }
        }

        current_region_number += 1;
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::I64(DataArray::from_vec(
            "RegionLabels",
            region_labels,
            1,
        )));
    pd.point_data_mut().set_active_scalars("RegionLabels");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_components() {
        let mut pd = PolyData::new();
        // Component A: chain of close points
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.5, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        // Component B: far away
        pd.points.push([100.0, 0.0, 0.0]);
        pd.points.push([100.5, 0.0, 0.0]);

        let result = connected_points(&pd, 0.6);
        let arr = result.point_data().get_array("RegionLabels").unwrap();
        let mut buf = [0.0f64];

        arr.tuple_as_f64(0, &mut buf);
        let c0 = buf[0] as i32;
        arr.tuple_as_f64(1, &mut buf);
        let c1 = buf[0] as i32;
        arr.tuple_as_f64(3, &mut buf);
        let c3 = buf[0] as i32;

        assert_eq!(c0, c1); // Same component
        assert_ne!(c0, c3); // Different component
    }

    #[test]
    fn single_component() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.5, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let result = connected_points(&pd, 0.6);
        let arr = result.point_data().get_array("RegionLabels").unwrap();
        let mut buf = [0.0f64];

        arr.tuple_as_f64(0, &mut buf);
        let c0 = buf[0] as i32;
        arr.tuple_as_f64(2, &mut buf);
        let c2 = buf[0] as i32;
        assert_eq!(c0, c2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = connected_points(&pd, 1.0);
        assert_eq!(result.points.len(), 0);
    }
}
