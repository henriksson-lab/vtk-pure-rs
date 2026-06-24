//! Compute deviation between vertex normal and average neighbor normal.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn normal_deviation(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let nm = compute_normals(mesh);
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
            }
        }
    }
    let data: Vec<f64> = (0..n)
        .map(|i| {
            if nb[i].is_empty() {
                return 0.0;
            }
            let ni = nm[i];
            let k = nb[i].len() as f64;
            let mut avg = [0.0, 0.0, 0.0];
            for &j in &nb[i] {
                avg[0] += nm[j][0];
                avg[1] += nm[j][1];
                avg[2] += nm[j][2];
            }
            avg[0] /= k;
            avg[1] /= k;
            avg[2] /= k;
            let dot = (ni[0] * avg[0] + ni[1] * avg[1] + ni[2] * avg[2]).clamp(-1.0, 1.0);
            dot.acos().to_degrees()
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalDeviation",
            data,
            1,
        )));
    r.point_data_mut().set_active_scalars("NormalDeviation");
    r
}
fn compute_normals(mesh: &PolyData) -> Vec<[f64; 3]> {
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
    fn test_flat() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = normal_deviation(&m);
        let mut buf = [0.0];
        r.point_data()
            .get_array("NormalDeviation")
            .unwrap()
            .tuple_as_f64(1, &mut buf);
        assert!(buf[0] < 5.0);
    } // flat mesh -> near zero deviation
    #[test]
    fn test_sharp() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let r = normal_deviation(&m);
        let arr = r.point_data().get_array("NormalDeviation").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 5.0);
    }
}
