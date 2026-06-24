//! Solve linear system using cotangent Laplacian (Poisson-like problems on mesh).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn cotan_poisson_solve(
    mesh: &PolyData,
    rhs_array: &str,
    boundary_conditions: &[(usize, f64)],
    iterations: usize,
) -> PolyData {
    let arr = match mesh.point_data().get_array(rhs_array) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let rhs: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let fixed: std::collections::HashMap<usize, f64> =
        boundary_conditions.iter().cloned().collect();
    // Build cotangent weights
    let mut nb_weights: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
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
            let cl = ((eij[1] * eik[2] - eij[2] * eik[1]).powi(2)
                + (eij[2] * eik[0] - eij[0] * eik[2]).powi(2)
                + (eij[0] * eik[1] - eij[1] * eik[0]).powi(2))
            .sqrt();
            let cot = if cl > 1e-15 {
                (dot / cl).abs() * 0.5
            } else {
                0.0
            };
            nb_weights[ids[j]].push((ids[k], cot));
            nb_weights[ids[k]].push((ids[j], cot));
        }
    }
    let mut u = vec![0.0f64; n];
    for (&i, &v) in &fixed {
        if i < n {
            u[i] = v;
        }
    }
    for _ in 0..iterations {
        let prev = u.clone();
        for i in 0..n {
            if fixed.contains_key(&i) || nb_weights[i].is_empty() {
                continue;
            }
            let mut wsum = 0.0;
            let mut vsum = 0.0;
            for &(j, w) in &nb_weights[i] {
                vsum += w * prev[j];
                wsum += w;
            }
            if wsum > 1e-15 {
                u[i] = (vsum + rhs[i]) / wsum;
            }
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Solution", u, 1)));
    r.point_data_mut().set_active_scalars("Solution");
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "rhs",
                vec![0.0, 0.0, 0.0, 0.0],
                1,
            )));
        let r = cotan_poisson_solve(&m, "rhs", &[(0, 0.0), (3, 1.0)], 50);
        let arr = r.point_data().get_array("Solution").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0].abs() < 0.1);
        arr.tuple_as_f64(3, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.1);
    }
}
