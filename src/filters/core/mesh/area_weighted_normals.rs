use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute area-weighted vertex normals for a triangle mesh.
///
/// This follows `vtkTriangleMeshPointNormals`: triangle normals are added
/// without first normalizing them, then point normals are normalized and
/// stored as the active point-data array named "Normals".
pub fn area_weighted_normals(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }
    if input.polys.iter().next().is_none() {
        return input.clone();
    }
    if input.polys.iter().any(|cell| cell.len() != 3) {
        return input.clone();
    }
    if input
        .polys
        .iter()
        .any(|cell| cell.iter().any(|&id| id < 0 || (id as usize) >= n))
    {
        return input.clone();
    }

    let mut normals = vec![[0.0f64; 3]; n];

    for cell in input.polys.iter() {
        let p0 = input.points.get(cell[0] as usize);
        let p1 = input.points.get(cell[1] as usize);
        let p2 = input.points.get(cell[2] as usize);
        let a = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
        let b = [p0[0] - p1[0], p0[1] - p1[1], p0[2] - p1[2]];
        let tri_normal = [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ];

        for &id in cell {
            let idx = id as usize;
            normals[idx][0] += tri_normal[0];
            normals[idx][1] += tri_normal[1];
            normals[idx][2] += tri_normal[2];
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
        .add_array(AnyDataArray::F64(DataArray::from_vec("Normals", flat, 3)));
    pd.point_data_mut().set_active_normals("Normals");
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
        if !cell.iter().all(|&id| id >= 0 && (id as usize) < n) {
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
        let arr = result.point_data().get_array("Normals").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[2] - 1.0).abs() < 1e-10); // Z-up normal
    }

    #[test]
    fn area_weighted_rejects_non_triangles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = area_weighted_normals(&pd);
        assert!(result.point_data().get_array("Normals").is_none());
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
