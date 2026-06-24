//! Per-face angle statistics (min/max/avg interior angle).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn face_min_angle_array(mesh: &PolyData) -> PolyData {
    let data: Vec<f64> = mesh
        .polys
        .iter()
        .map(|cell| {
            if cell.len() != 3 {
                return 0.0;
            }
            let p = [
                mesh.points.get(cell[0] as usize),
                mesh.points.get(cell[1] as usize),
                mesh.points.get(cell[2] as usize),
            ];
            let mut mn = 180.0f64;
            for i in 0..3 {
                let v1 = [
                    p[(i + 1) % 3][0] - p[i][0],
                    p[(i + 1) % 3][1] - p[i][1],
                    p[(i + 1) % 3][2] - p[i][2],
                ];
                let v2 = [
                    p[(i + 2) % 3][0] - p[i][0],
                    p[(i + 2) % 3][1] - p[i][1],
                    p[(i + 2) % 3][2] - p[i][2],
                ];
                let d = v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2];
                let l1 = (v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]).sqrt();
                let l2 = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();
                if l1 > 1e-15 && l2 > 1e-15 {
                    mn = mn.min((d / (l1 * l2)).clamp(-1.0, 1.0).acos().to_degrees());
                }
            }
            mn
        })
        .collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("MinAngle", data, 1)));
    r
}
pub fn face_max_angle_array(mesh: &PolyData) -> PolyData {
    let data: Vec<f64> = mesh
        .polys
        .iter()
        .map(|cell| {
            if cell.len() != 3 {
                return 0.0;
            }
            let p = [
                mesh.points.get(cell[0] as usize),
                mesh.points.get(cell[1] as usize),
                mesh.points.get(cell[2] as usize),
            ];
            let mut mx = 0.0f64;
            for i in 0..3 {
                let v1 = [
                    p[(i + 1) % 3][0] - p[i][0],
                    p[(i + 1) % 3][1] - p[i][1],
                    p[(i + 1) % 3][2] - p[i][2],
                ];
                let v2 = [
                    p[(i + 2) % 3][0] - p[i][0],
                    p[(i + 2) % 3][1] - p[i][1],
                    p[(i + 2) % 3][2] - p[i][2],
                ];
                let d = v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2];
                let l1 = (v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]).sqrt();
                let l2 = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();
                if l1 > 1e-15 && l2 > 1e-15 {
                    mx = mx.max((d / (l1 * l2)).clamp(-1.0, 1.0).acos().to_degrees());
                }
            }
            mx
        })
        .collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("MaxAngle", data, 1)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_equilateral() {
        let h = 3.0f64.sqrt() / 2.0;
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, h, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = face_min_angle_array(&m);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("MinAngle")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 60.0).abs() < 1.0);
    }
}
