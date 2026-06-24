//! Compute min/max/mean interior angles per face as cell data.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn edge_angle_stats(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut min_angles = Vec::new();
    let mut max_angles = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            min_angles.push(0.0);
            max_angles.push(0.0);
            continue;
        }
        let nc = cell.len();
        let mut face_min = std::f64::consts::PI;
        let mut face_max = 0.0f64;
        for i in 0..nc {
            let vi = cell[i] as usize;
            let vp = cell[(i + nc - 1) % nc] as usize;
            let vn = cell[(i + 1) % nc] as usize;
            if vi >= n || vp >= n || vn >= n {
                continue;
            }
            let p = mesh.points.get(vi);
            let pp = mesh.points.get(vp);
            let pn = mesh.points.get(vn);
            let u = [pp[0] - p[0], pp[1] - p[1], pp[2] - p[2]];
            let v = [pn[0] - p[0], pn[1] - p[1], pn[2] - p[2]];
            let lu = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
            let lv = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            if lu < 1e-15 || lv < 1e-15 {
                continue;
            }
            let cos_a = (u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) / (lu * lv);
            let angle = cos_a.clamp(-1.0, 1.0).acos();
            face_min = face_min.min(angle);
            face_max = face_max.max(angle);
        }
        min_angles.push(face_min * 180.0 / std::f64::consts::PI);
        max_angles.push(face_max * 180.0 / std::f64::consts::PI);
    }
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MinAngle", min_angles, 1,
        )));
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MaxAngle", max_angles, 1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_angle_stats() {
        // Equilateral: all angles = 60 degrees
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 3.0f64.sqrt() / 2.0, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = edge_angle_stats(&mesh);
        let min_arr = r.cell_data().get_array("MinAngle").unwrap();
        let mut b = [0.0f64];
        min_arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 60.0).abs() < 1.0);
    }
}
