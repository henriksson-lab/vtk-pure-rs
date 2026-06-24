//! Compute an isolation score for each vertex (average distance to neighbors).
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn isolation_score(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !adj[a].contains(&b) {
                    adj[a].push(b);
                }
                if !adj[b].contains(&a) {
                    adj[b].push(a);
                }
            }
        }
    }
    let scores: Vec<f64> = (0..n)
        .map(|i| {
            if adj[i].is_empty() {
                return f64::INFINITY;
            }
            let p = mesh.points.get(i);
            let avg: f64 = adj[i]
                .iter()
                .map(|&j| {
                    let q = mesh.points.get(j);
                    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
                })
                .sum::<f64>()
                / adj[i].len() as f64;
            avg
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "IsolationScore",
            scores,
            1,
        )));
    result.point_data_mut().set_active_scalars("IsolationScore");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_isolation() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = isolation_score(&mesh);
        let arr = r.point_data().get_array("IsolationScore").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!(b[0] > 0.5); // average edge length > 0.5
    }
}
