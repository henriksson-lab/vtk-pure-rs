use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the angle at each vertex within each triangle.
///
/// Adds "MinVertexAngle" and "MaxVertexAngle" point data arrays
/// (min and max angle at each vertex across all its triangles, in degrees).
pub fn vertex_angle_extremes(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut min_angle = vec![180.0f64; n];
    let mut max_angle = vec![0.0f64; n];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let pts: Vec<[f64; 3]> = cell
            .iter()
            .map(|&id| input.points.get(id as usize))
            .collect();
        let nc = pts.len();

        for i in 0..nc {
            let prev = pts[(i + nc - 1) % nc];
            let cur = pts[i];
            let next = pts[(i + 1) % nc];
            let e1 = [prev[0] - cur[0], prev[1] - cur[1], prev[2] - cur[2]];
            let e2 = [next[0] - cur[0], next[1] - cur[1], next[2] - cur[2]];
            let l1 = (e1[0] * e1[0] + e1[1] * e1[1] + e1[2] * e1[2]).sqrt();
            let l2 = (e2[0] * e2[0] + e2[1] * e2[1] + e2[2] * e2[2]).sqrt();
            if l1 > 1e-15 && l2 > 1e-15 {
                let cos_a = (e1[0] * e2[0] + e1[1] * e2[1] + e1[2] * e2[2]) / (l1 * l2);
                let angle = cos_a.clamp(-1.0, 1.0).acos().to_degrees();
                let vi = cell[i] as usize;
                min_angle[vi] = min_angle[vi].min(angle);
                max_angle[vi] = max_angle[vi].max(angle);
            }
        }
    }

    // Fix unvisited vertices
    for i in 0..n {
        if max_angle[i] == 0.0 {
            min_angle[i] = 0.0;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MinVertexAngle",
            min_angle,
            1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MaxVertexAngle",
            max_angle,
            1,
        )));
    pd
}

/// Compute overall min and max angles across the entire mesh.
pub fn mesh_angle_range(input: &PolyData) -> (f64, f64) {
    let result = vertex_angle_extremes(input);
    let min_a = match result.point_data().get_array("MinVertexAngle") {
        Some(a) => a,
        None => return (0.0, 0.0),
    };
    let max_a = result.point_data().get_array("MaxVertexAngle").unwrap();
    let n = min_a.num_tuples();
    let mut buf = [0.0f64];
    let mut global_min: f64 = 180.0;
    let mut global_max: f64 = 0.0;
    for i in 0..n {
        max_a.tuple_as_f64(i, &mut buf);
        if buf[0] > 0.0 {
            global_max = global_max.max(buf[0]);
        }
        min_a.tuple_as_f64(i, &mut buf);
        if buf[0] > 0.0 {
            global_min = global_min.min(buf[0]);
        }
    }
    if global_max == 0.0 {
        global_min = 0.0;
    }
    (global_min, global_max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equilateral_60() {
        let h = (3.0f64).sqrt() / 2.0;
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, h, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = vertex_angle_extremes(&pd);
        let min_a = result.point_data().get_array("MinVertexAngle").unwrap();
        let mut buf = [0.0f64];
        min_a.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 60.0).abs() < 1.0);
    }

    #[test]
    fn right_angle_range() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let (min, max) = mesh_angle_range(&pd);
        assert!(min < 50.0); // ~45 degrees
        assert!(max > 85.0); // ~90 degrees
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let (min, max) = mesh_angle_range(&pd);
        assert_eq!(min + max, 0.0);
    }
}
