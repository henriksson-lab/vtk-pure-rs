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
        let Some(ids) = valid_triangle_ids(c, n) else {
            continue;
        };
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
fn valid_triangle_ids(cell: &[i64], n_points: usize) -> Option<[usize; 3]> {
    let a = usize::try_from(cell[0]).ok()?;
    let b = usize::try_from(cell[1]).ok()?;
    let c = usize::try_from(cell[2]).ok()?;
    if a >= n_points || b >= n_points || c >= n_points || a == b || b == c || c == a {
        return None;
    }
    Some([a, b, c])
}
pub fn attach_degree_weight(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut deg = vec![0.0f64; n];
    let mut seen: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.lines.iter() {
        for pair in cell.windows(2) {
            insert_degree_edge(n, &mut seen, pair[0], pair[1]);
        }
    }
    for cell in mesh.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            insert_degree_edge(n, &mut seen, cell[i], cell[(i + 1) % cell.len()]);
        }
    }
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            insert_degree_edge(n, &mut seen, tri[0], tri[1]);
            insert_degree_edge(n, &mut seen, tri[1], tri[2]);
            insert_degree_edge(n, &mut seen, tri[2], tri[0]);
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
fn insert_degree_edge(n: usize, seen: &mut [std::collections::HashSet<usize>], a: i64, b: i64) {
    let (Ok(a), Ok(b)) = (usize::try_from(a), usize::try_from(b)) else {
        return;
    };
    if a >= n || b >= n || a == b {
        return;
    }
    seen[a].insert(b);
    seen[b].insert(a);
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
