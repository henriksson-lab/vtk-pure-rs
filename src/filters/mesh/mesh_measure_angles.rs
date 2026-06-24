//! Measure interior angles of mesh triangles.
use crate::data::{AnyDataArray, DataArray, PolyData};
/// Compute minimum interior angle per triangle (degrees).
pub fn min_angles(mesh: &PolyData) -> PolyData {
    let mut data = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() != 3 {
            data.push(0.0);
            continue;
        }
        let p = [
            mesh.points.get(cell[0] as usize),
            mesh.points.get(cell[1] as usize),
            mesh.points.get(cell[2] as usize),
        ];
        let mut min_a = 180.0f64;
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
                min_a = min_a.min((d / (l1 * l2)).clamp(-1.0, 1.0).acos().to_degrees());
            }
        }
        data.push(min_a);
    }
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("MinAngle", data, 1)));
    r
}
/// Compute maximum interior angle per triangle (degrees).
pub fn max_angles(mesh: &PolyData) -> PolyData {
    let mut data = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() != 3 {
            data.push(0.0);
            continue;
        }
        let p = [
            mesh.points.get(cell[0] as usize),
            mesh.points.get(cell[1] as usize),
            mesh.points.get(cell[2] as usize),
        ];
        let mut max_a = 0.0f64;
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
                max_a = max_a.max((d / (l1 * l2)).clamp(-1.0, 1.0).acos().to_degrees());
            }
        }
        data.push(max_a);
    }
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
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, h, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = min_angles(&mesh);
        let arr = r.cell_data().get_array("MinAngle").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 60.0).abs() < 1.0);
    }
}
