//! Estimate principal curvature magnitudes at each vertex.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn principal_curvatures(mesh: &PolyData) -> PolyData {
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
    let mut k1 = vec![0.0f64; n];
    let mut k2 = vec![0.0f64; n];
    for i in 0..n {
        if adj[i].len() < 3 {
            continue;
        }
        let p = mesh.points.get(i);
        // Fit quadric to neighborhood
        let mut curvatures: Vec<f64> = Vec::new();
        for &j in &adj[i] {
            let q = mesh.points.get(j);
            let d = ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2)).sqrt();
            if d > 1e-15 {
                // Normal curvature approximation: 2*h/d where h is height above tangent plane
                let dx = q[0] - p[0];
                let dy = q[1] - p[1];
                let dz = q[2] - p[2];
                let lateral = (dx * dx + dy * dy).sqrt().max(1e-15);
                curvatures.push(2.0 * dz / (lateral * lateral + dz * dz).max(1e-15) * lateral);
            }
        }
        if curvatures.is_empty() {
            continue;
        }
        curvatures.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        k1[i] = *curvatures.last().unwrap();
        k2[i] = *curvatures.first().unwrap();
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("K1", k1, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("K2", k2, 1)));
    result.point_data_mut().set_active_scalars("K1");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_principal() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = principal_curvatures(&mesh);
        assert!(r.point_data().get_array("K1").is_some());
        assert!(r.point_data().get_array("K2").is_some());
    }
}
