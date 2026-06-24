//! Scale mesh from each vertex's perspective (inflate/deflate).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn inflate(mesh: &PolyData, amount: f64) -> PolyData {
    let n = mesh.points.len();
    let nm = calc_nm(mesh);
    let mut r = mesh.clone();
    for i in 0..n {
        let p = r.points.get(i);
        r.points.set(
            i,
            [
                p[0] + nm[i][0] * amount,
                p[1] + nm[i][1] * amount,
                p[2] + nm[i][2] * amount,
            ],
        );
    }
    r
}
pub fn deflate(mesh: &PolyData, amount: f64) -> PolyData {
    inflate(mesh, -amount)
}
pub fn scale_along_normals_by_scalar(mesh: &PolyData, array_name: &str, scale: f64) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    let nm = calc_nm(mesh);
    let mut r = mesh.clone();
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let d = buf[0] * scale;
        let p = r.points.get(i);
        r.points.set(
            i,
            [
                p[0] + nm[i][0] * d,
                p[1] + nm[i][1] * d,
                p[2] + nm[i][2] * d,
            ],
        );
    }
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
    fn test_inflate() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = inflate(&m, 0.1);
        let p = r.points.get(0);
        assert!(p[2].abs() > 0.05);
    }
    #[test]
    fn test_deflate() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = deflate(&m, 0.1);
        let p = r.points.get(0);
        assert!(p[2].abs() > 0.05);
    }
}
