//! Estimate tangent plane at each vertex using PCA of 1-ring neighborhood.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn tangent_plane(mesh: &PolyData) -> PolyData {
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
    let mut planarity = vec![0.0f64; n];
    for i in 0..n {
        if adj[i].len() < 3 {
            planarity[i] = 1.0;
            continue;
        }
        let p = mesh.points.get(i);
        let k = adj[i].len() as f64;
        // Centroid of neighborhood
        let mut cx = p[0];
        let mut cy = p[1];
        let mut cz = p[2];
        for &j in &adj[i] {
            let q = mesh.points.get(j);
            cx += q[0];
            cy += q[1];
            cz += q[2];
        }
        cx /= k + 1.0;
        cy /= k + 1.0;
        cz /= k + 1.0;
        // Covariance matrix (3x3)
        let mut cov = [[0.0f64; 3]; 3];
        let mut add_point = |px: f64, py: f64, pz: f64| {
            let d = [px - cx, py - cy, pz - cz];
            for r in 0..3 {
                for c in 0..3 {
                    cov[r][c] += d[r] * d[c];
                }
            }
        };
        add_point(p[0], p[1], p[2]);
        for &j in &adj[i] {
            let q = mesh.points.get(j);
            add_point(q[0], q[1], q[2]);
        }
        // Trace and smallest eigenvalue approximation via characteristic equation
        let trace = cov[0][0] + cov[1][1] + cov[2][2];
        let det = cov[0][0] * (cov[1][1] * cov[2][2] - cov[1][2] * cov[2][1])
            - cov[0][1] * (cov[1][0] * cov[2][2] - cov[1][2] * cov[2][0])
            + cov[0][2] * (cov[1][0] * cov[2][1] - cov[1][1] * cov[2][0]);
        // Planarity ~ smallest eigenvalue / trace
        planarity[i] = if trace > 1e-15 {
            1.0 - det.abs().powf(1.0 / 3.0) / (trace / 3.0)
        } else {
            1.0
        };
        planarity[i] = planarity[i].clamp(0.0, 1.0);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Planarity",
            planarity,
            1,
        )));
    result.point_data_mut().set_active_scalars("Planarity");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tangent() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = tangent_plane(&mesh);
        assert!(r.point_data().get_array("Planarity").is_some());
    }
}
