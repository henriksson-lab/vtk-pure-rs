//! Compute deviation between each face normal and its neighbors' normals.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn face_normal_deviation(mesh: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    let fnorms: Vec<[f64; 3]> = cells.iter().map(|c| fnorm(c, mesh)).collect();
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, c) in cells.iter().enumerate() {
        let n = c.len();
        for i in 0..n {
            let a = c[i] as usize;
            let b = c[(i + 1) % n] as usize;
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); nc];
    for (_, faces) in &ef {
        for i in 0..faces.len() {
            for j in i + 1..faces.len() {
                adj[faces[i]].push(faces[j]);
                adj[faces[j]].push(faces[i]);
            }
        }
    }
    let data: Vec<f64> = (0..nc)
        .map(|ci| {
            if adj[ci].is_empty() {
                return 0.0;
            }
            let n0 = fnorms[ci];
            let mut max_dev = 0.0f64;
            for &ni in &adj[ci] {
                let n1 = fnorms[ni];
                let dot = (n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2]).clamp(-1.0, 1.0);
                max_dev = max_dev.max(dot.acos().to_degrees());
            }
            max_dev
        })
        .collect();
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalDeviation",
            data,
            1,
        )));
    r
}
fn fnorm(c: &[i64], m: &PolyData) -> [f64; 3] {
    if c.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let a = m.points.get(c[0] as usize);
    let b = m.points.get(c[1] as usize);
    let cc = m.points.get(c[2] as usize);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [cc[0] - a[0], cc[1] - a[1], cc[2] - a[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if l < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [n[0] / l, n[1] / l, n[2] / l]
    }
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
        let r = face_normal_deviation(&m);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("NormalDeviation")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 1.0);
    }
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
        let r = face_normal_deviation(&m);
        let mut buf = [0.0];
        r.cell_data()
            .get_array("NormalDeviation")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 10.0);
    }
}
