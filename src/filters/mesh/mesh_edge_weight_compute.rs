//! Compute various edge weight schemes (uniform, cotangent, distance).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn attach_cotangent_weights(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut weight_sum = vec![0.0f64; n];
    for c in &cells {
        if c.len() != 3 {
            continue;
        }
        let ids = [c[0] as usize, c[1] as usize, c[2] as usize];
        let p = [
            mesh.points.get(ids[0]),
            mesh.points.get(ids[1]),
            mesh.points.get(ids[2]),
        ];
        for i in 0..3 {
            let j = (i + 1) % 3;
            let k = (i + 2) % 3;
            let eij = [p[j][0] - p[i][0], p[j][1] - p[i][1], p[j][2] - p[i][2]];
            let eik = [p[k][0] - p[i][0], p[k][1] - p[i][1], p[k][2] - p[i][2]];
            let dot = eij[0] * eik[0] + eij[1] * eik[1] + eij[2] * eik[2];
            let cross_l = ((eij[1] * eik[2] - eij[2] * eik[1]).powi(2)
                + (eij[2] * eik[0] - eij[0] * eik[2]).powi(2)
                + (eij[0] * eik[1] - eij[1] * eik[0]).powi(2))
            .sqrt();
            let cot = if cross_l > 1e-15 {
                (dot / cross_l).abs()
            } else {
                0.0
            };
            weight_sum[ids[j]] += cot;
            weight_sum[ids[k]] += cot;
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CotWeight",
            weight_sum,
            1,
        )));
    r
}
pub fn attach_degree_weight(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut deg = vec![0.0f64; n];
    let mut seen: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                seen[a].insert(b);
                seen[b].insert(a);
            }
        }
    }
    for i in 0..n {
        deg[i] = seen[i].len() as f64;
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Degree", deg, 1)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cot() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = attach_cotangent_weights(&m);
        assert!(r.point_data().get_array("CotWeight").is_some());
    }
    #[test]
    fn test_deg() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = attach_degree_weight(&m);
        let mut buf = [0.0];
        r.point_data()
            .get_array("Degree")
            .unwrap()
            .tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 3.0);
    }
}
