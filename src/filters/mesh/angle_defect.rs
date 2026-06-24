use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the angle defect at each vertex.
///
/// Angle defect = 2π - sum(incident_angles). Related to discrete
/// Gaussian curvature by the Gauss-Bonnet theorem. Adds "AngleDefect".
pub fn angle_defect(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut angle_sum = vec![0.0f64; n];

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
            let prev = &pts[(i + nc - 1) % nc];
            let cur = &pts[i];
            let next = &pts[(i + 1) % nc];
            let e1 = [prev[0] - cur[0], prev[1] - cur[1], prev[2] - cur[2]];
            let e2 = [next[0] - cur[0], next[1] - cur[1], next[2] - cur[2]];
            let dot = e1[0] * e2[0] + e1[1] * e2[1] + e1[2] * e2[2];
            let l1 = (e1[0] * e1[0] + e1[1] * e1[1] + e1[2] * e1[2]).sqrt();
            let l2 = (e2[0] * e2[0] + e2[1] * e2[1] + e2[2] * e2[2]).sqrt();
            if l1 > 1e-15 && l2 > 1e-15 {
                angle_sum[cell[i] as usize] += (dot / (l1 * l2)).clamp(-1.0, 1.0).acos();
            }
        }
    }

    let defect: Vec<f64> = angle_sum
        .iter()
        .map(|&s| {
            if s > 0.0 {
                2.0 * std::f64::consts::PI - s
            } else {
                0.0
            }
        })
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AngleDefect",
            defect,
            1,
        )));
    pd
}

/// Verify the discrete Gauss-Bonnet theorem: sum of angle defects = 2π * χ.
pub fn gauss_bonnet_check(input: &PolyData) -> f64 {
    let result = angle_defect(input);
    let arr = match result.point_data().get_array("AngleDefect") {
        Some(a) => a,
        None => return 0.0,
    };
    let mut buf = [0.0f64];
    let mut total = 0.0;
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        total += buf[0];
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_triangle_defect() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = angle_defect(&pd);
        assert!(result.point_data().get_array("AngleDefect").is_some());
    }

    #[test]
    fn gauss_bonnet_tetrahedron() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 1]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let total = gauss_bonnet_check(&pd);
        // Should be ~4π for a closed surface with χ=2
        assert!((total - 4.0 * std::f64::consts::PI).abs() < 1.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(gauss_bonnet_check(&pd), 0.0);
    }
}
