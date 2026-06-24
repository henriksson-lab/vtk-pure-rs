use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute signed angles between consecutive edges at each vertex.
///
/// For polygon boundary vertices, computes the turning angle.
/// Adds "TurningAngle" point data in radians (positive = left turn).
pub fn turning_angles(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut angles = vec![0.0f64; n];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let nc = cell.len();
        for i in 0..nc {
            let prev = input.points.get(cell[(i + nc - 1) % nc] as usize);
            let cur = input.points.get(cell[i] as usize);
            let next = input.points.get(cell[(i + 1) % nc] as usize);

            let e1 = [cur[0] - prev[0], cur[1] - prev[1], cur[2] - prev[2]];
            let e2 = [next[0] - cur[0], next[1] - cur[1], next[2] - cur[2]];

            // Cross product Z component for 2D angle
            let cross_z = e1[0] * e2[1] - e1[1] * e2[0];
            let dot = e1[0] * e2[0] + e1[1] * e2[1] + e1[2] * e2[2];
            let l1 = (e1[0] * e1[0] + e1[1] * e1[1] + e1[2] * e1[2]).sqrt();
            let l2 = (e2[0] * e2[0] + e2[1] * e2[1] + e2[2] * e2[2]).sqrt();

            if l1 > 1e-15 && l2 > 1e-15 {
                let cos_a = (dot / (l1 * l2)).clamp(-1.0, 1.0);
                let angle = cos_a.acos();
                angles[cell[i] as usize] += if cross_z >= 0.0 { angle } else { -angle };
            }
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "TurningAngle",
            angles,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_angles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = turning_angles(&pd);
        assert!(result.point_data().get_array("TurningAngle").is_some());
    }

    #[test]
    fn right_angle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = turning_angles(&pd);
        let arr = result.point_data().get_array("TurningAngle").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0].abs() - std::f64::consts::FRAC_PI_2).abs() < 0.1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = turning_angles(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
