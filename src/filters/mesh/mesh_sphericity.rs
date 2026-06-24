//! Compute sphericity (ratio of inscribed to circumscribed sphere radii) at each vertex neighborhood.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn vertex_sphericity(mesh: &PolyData) -> PolyData {
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
    let sphericity: Vec<f64> = (0..n)
        .map(|i| {
            if adj[i].len() < 3 {
                return 0.0;
            }
            let p = mesh.points.get(i);
            let mut min_d = f64::INFINITY;
            let mut max_d = 0.0f64;
            for &j in &adj[i] {
                let q = mesh.points.get(j);
                let d =
                    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
                if d < min_d {
                    min_d = d;
                }
                if d > max_d {
                    max_d = d;
                }
            }
            if max_d > 1e-15 {
                min_d / max_d
            } else {
                1.0
            }
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Sphericity",
            sphericity,
            1,
        )));
    result.point_data_mut().set_active_scalars("Sphericity");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sphericity() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = vertex_sphericity(&mesh);
        assert!(r.point_data().get_array("Sphericity").is_some());
    }
}
