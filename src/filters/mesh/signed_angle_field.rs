use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the signed angle field: angle from a reference direction projected onto the tangent plane.
///
/// For each vertex, projects a reference direction onto the vertex tangent plane
/// and computes the angle from the local X axis. Adds "SignedAngleField" scalar.
/// Useful for direction fields and parameterization.
pub fn signed_angle_field(input: &PolyData, reference: [f64; 3]) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Compute vertex normals
    let mut vnormals = vec![[0.0f64; 3]; n];
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &id in cell.iter() {
            let i = id as usize;
            vnormals[i][0] += fn_[0];
            vnormals[i][1] += fn_[1];
            vnormals[i][2] += fn_[2];
        }
    }
    for nm in &mut vnormals {
        let l = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
        if l > 1e-15 {
            nm[0] /= l;
            nm[1] /= l;
            nm[2] /= l;
        }
    }

    let mut angles = vec![0.0f64; n];
    for i in 0..n {
        let nm = vnormals[i];
        // Project reference onto tangent plane
        let dot = reference[0] * nm[0] + reference[1] * nm[1] + reference[2] * nm[2];
        let proj = [
            reference[0] - dot * nm[0],
            reference[1] - dot * nm[1],
            reference[2] - dot * nm[2],
        ];
        let pl = (proj[0] * proj[0] + proj[1] * proj[1] + proj[2] * proj[2]).sqrt();
        if pl < 1e-15 {
            continue;
        }

        // Compute local X axis
        let up = if nm[2].abs() < 0.9 {
            [0.0, 0.0, 1.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        let tx = [
            nm[1] * up[2] - nm[2] * up[1],
            nm[2] * up[0] - nm[0] * up[2],
            nm[0] * up[1] - nm[1] * up[0],
        ];
        let tl = (tx[0] * tx[0] + tx[1] * tx[1] + tx[2] * tx[2]).sqrt();
        if tl < 1e-15 {
            continue;
        }
        let tx = [tx[0] / tl, tx[1] / tl, tx[2] / tl];
        let ty = [
            nm[1] * tx[2] - nm[2] * tx[1],
            nm[2] * tx[0] - nm[0] * tx[2],
            nm[0] * tx[1] - nm[1] * tx[0],
        ];

        let px = proj[0] * tx[0] + proj[1] * tx[1] + proj[2] * tx[2];
        let py = proj[0] * ty[0] + proj[1] * ty[1] + proj[2] * ty[2];
        angles[i] = py.atan2(px);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SignedAngleField",
            angles,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_field_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = signed_angle_field(&pd, [1.0, 0.0, 0.0]);
        assert!(result.point_data().get_array("SignedAngleField").is_some());
    }

    #[test]
    fn consistent_on_flat() {
        let mut pd = PolyData::new();
        for j in 0..3 {
            for i in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..2 {
            for i in 0..2 {
                let a = (j * 3 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 4]);
                pd.polys.push_cell(&[a, a + 4, a + 3]);
            }
        }

        let result = signed_angle_field(&pd, [1.0, 0.0, 0.0]);
        let arr = result.point_data().get_array("SignedAngleField").unwrap();
        let mut buf = [0.0f64];
        // All normals = +Z, reference = +X -> all angles should be similar
        arr.tuple_as_f64(0, &mut buf);
        let a0 = buf[0];
        arr.tuple_as_f64(4, &mut buf);
        let a4 = buf[0];
        assert!((a0 - a4).abs() < 0.1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = signed_angle_field(&pd, [1.0, 0.0, 0.0]);
        assert_eq!(result.points.len(), 0);
    }
}
