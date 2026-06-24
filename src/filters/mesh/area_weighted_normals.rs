use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute area-weighted vertex normals.
///
/// Unlike simple averaging, each face normal's contribution is weighted
/// by the face area. Produces more accurate normals for meshes with
/// varying triangle sizes. Adds "AreaWeightedNormals" array.
pub fn area_weighted_normals(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut normals = vec![[0.0f64; 3]; n];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            // Cross product = area-weighted normal (no normalization)
            let fn_ = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];

            for &id in &[cell[0], cell[i], cell[i + 1]] {
                let idx = id as usize;
                normals[idx][0] += fn_[0];
                normals[idx][1] += fn_[1];
                normals[idx][2] += fn_[2];
            }
        }
    }

    // Normalize
    let flat: Vec<f64> = normals
        .iter()
        .flat_map(|n| {
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if l > 1e-15 {
                vec![n[0] / l, n[1] / l, n[2] / l]
            } else {
                vec![0.0, 0.0, 0.0]
            }
        })
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AreaWeightedNormals",
            flat,
            3,
        )));
    pd.point_data_mut()
        .set_active_normals("AreaWeightedNormals");
    pd
}

/// Compute angle-weighted vertex normals (Thürmer-Wüthrich method).
///
/// Each face normal is weighted by the angle at that vertex.
pub fn angle_weighted_normals(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut normals = vec![[0.0f64; 3]; n];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let pts: Vec<[f64; 3]> = cell
            .iter()
            .map(|&id| input.points.get(id as usize))
            .collect();
        let nc = pts.len();

        // Face normal
        let e1 = [
            pts[1][0] - pts[0][0],
            pts[1][1] - pts[0][1],
            pts[1][2] - pts[0][2],
        ];
        let e2 = [
            pts[2][0] - pts[0][0],
            pts[2][1] - pts[0][1],
            pts[2][2] - pts[0][2],
        ];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let fl = (fn_[0] * fn_[0] + fn_[1] * fn_[1] + fn_[2] * fn_[2]).sqrt();
        if fl < 1e-15 {
            continue;
        }
        let fn_n = [fn_[0] / fl, fn_[1] / fl, fn_[2] / fl];

        for i in 0..nc {
            let prev = pts[(i + nc - 1) % nc];
            let cur = pts[i];
            let next = pts[(i + 1) % nc];
            let ea = [prev[0] - cur[0], prev[1] - cur[1], prev[2] - cur[2]];
            let eb = [next[0] - cur[0], next[1] - cur[1], next[2] - cur[2]];
            let la = (ea[0] * ea[0] + ea[1] * ea[1] + ea[2] * ea[2]).sqrt();
            let lb = (eb[0] * eb[0] + eb[1] * eb[1] + eb[2] * eb[2]).sqrt();
            if la > 1e-15 && lb > 1e-15 {
                let cos_a = (ea[0] * eb[0] + ea[1] * eb[1] + ea[2] * eb[2]) / (la * lb);
                let angle = cos_a.clamp(-1.0, 1.0).acos();
                let idx = cell[i] as usize;
                normals[idx][0] += angle * fn_n[0];
                normals[idx][1] += angle * fn_n[1];
                normals[idx][2] += angle * fn_n[2];
            }
        }
    }

    let flat: Vec<f64> = normals
        .iter()
        .flat_map(|n| {
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if l > 1e-15 {
                vec![n[0] / l, n[1] / l, n[2] / l]
            } else {
                vec![0.0; 3]
            }
        })
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AngleWeightedNormals",
            flat,
            3,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_weighted() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = area_weighted_normals(&pd);
        let arr = result
            .point_data()
            .get_array("AreaWeightedNormals")
            .unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[2] - 1.0).abs() < 1e-10); // Z-up normal
    }

    #[test]
    fn angle_weighted() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = angle_weighted_normals(&pd);
        let arr = result
            .point_data()
            .get_array("AngleWeightedNormals")
            .unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[2] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let r1 = area_weighted_normals(&pd);
        let r2 = angle_weighted_normals(&pd);
        assert_eq!(r1.points.len(), 0);
        assert_eq!(r2.points.len(), 0);
    }
}
