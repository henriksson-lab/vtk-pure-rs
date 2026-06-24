use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::collections::VecDeque;

/// Connected components for point clouds using epsilon-distance.
///
/// Two points are connected if their Euclidean distance is less than `epsilon`.
/// BFS flood-fill assigns each point a "ComponentId".
///
/// Returns a PolyData with vertex cells and a "ComponentId" point data array.
pub fn connected_points(input: &PolyData, epsilon: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return PolyData::new();
    }

    let eps2 = epsilon * epsilon;
    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    let mut component_ids = vec![-1i32; n];
    let mut current_component = 0i32;

    for start in 0..n {
        if component_ids[start] >= 0 {
            continue;
        }

        let mut queue = VecDeque::new();
        queue.push_back(start);
        component_ids[start] = current_component;

        while let Some(idx) = queue.pop_front() {
            let p = pts[idx];
            for j in 0..n {
                if component_ids[j] >= 0 {
                    continue;
                }
                let q = pts[j];
                let dx = p[0] - q[0];
                let dy = p[1] - q[1];
                let dz = p[2] - q[2];
                if dx * dx + dy * dy + dz * dz < eps2 {
                    component_ids[j] = current_component;
                    queue.push_back(j);
                }
            }
        }

        current_component += 1;
    }

    let mut out_points = Points::<f64>::new();
    let mut out_verts = CellArray::new();
    for i in 0..n {
        out_points.push(pts[i]);
        out_verts.push_cell(&[i as i64]);
    }

    let comp_f64: Vec<f64> = component_ids.iter().map(|&c| c as f64).collect();

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ComponentId",
            comp_f64,
            1,
        )));
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
        let arr = result.point_data().get_array("ComponentId").unwrap();
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
        let arr = result.point_data().get_array("ComponentId").unwrap();
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
