//! Compute angle between vertex normal and a reference axis.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn normal_angle_to_z(mesh: &PolyData) -> PolyData {
    normal_angle_to_axis(mesh, [0.0, 0.0, 1.0], "NormalAngleZ")
}
pub fn normal_angle_to_y(mesh: &PolyData) -> PolyData {
    normal_angle_to_axis(mesh, [0.0, 1.0, 0.0], "NormalAngleY")
}
pub fn normal_angle_to_x(mesh: &PolyData) -> PolyData {
    normal_angle_to_axis(mesh, [1.0, 0.0, 0.0], "NormalAngleX")
}
pub fn normal_angle_to_axis(mesh: &PolyData, axis: [f64; 3], name: &str) -> PolyData {
    let n = mesh.points.len();
    let nm = calc_nm(mesh);
    let al = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2])
        .sqrt()
        .max(1e-15);
    let a = [axis[0] / al, axis[1] / al, axis[2] / al];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let dot = (nm[i][0] * a[0] + nm[i][1] * a[1] + nm[i][2] * a[2]).clamp(-1.0, 1.0);
            dot.acos().to_degrees()
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(name, data, 1)));
    r.point_data_mut().set_active_scalars(name);
    r
}
fn calc_nm(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &v in cell {
            let vi = v as usize;
            if vi < n {
                nm[vi][0] += fn_[0];
                nm[vi][1] += fn_[1];
                nm[vi][2] += fn_[2];
            }
        }
    }
    for v in &mut nm {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 1e-15 {
            v[0] /= l;
            v[1] /= l;
            v[2] /= l;
        }
    }
    nm
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_z() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = normal_angle_to_z(&m);
        let mut buf = [0.0];
        r.point_data()
            .get_array("NormalAngleZ")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 5.0 || buf[0] > 175.0);
    } // flat in XY -> normal ~parallel to Z
    #[test]
    fn test_custom() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = normal_angle_to_axis(&m, [1.0, 0.0, 0.0], "test");
        assert!(r.point_data().get_array("test").is_some());
    }
}
