//! Curvature tensor analysis: mean/Gaussian curvature, shape operator eigenvalues.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute mean and Gaussian curvature at each vertex.
pub fn curvature_analysis(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let mut mean_curv = Vec::with_capacity(n);
    let mut gauss_curv = Vec::with_capacity(n);

    for i in 0..n {
        if adj[i].is_empty() {
            mean_curv.push(0.0);
            gauss_curv.push(0.0);
            continue;
        }
        let p = mesh.points.get(i);
        // Mean curvature via Laplacian
        let mut lap = [0.0; 3];
        for &j in &adj[i] {
            let q = mesh.points.get(j);
            for c in 0..3 {
                lap[c] += q[c] - p[c];
            }
        }
        let k = adj[i].len() as f64;
        let h = ((lap[0] / k).powi(2) + (lap[1] / k).powi(2) + (lap[2] / k).powi(2)).sqrt();
        mean_curv.push(h);

        // Gaussian curvature via angle defect
        let mut angle_sum = 0.0;
        let neighbors: Vec<usize> = adj[i].clone();
        for ni in 0..neighbors.len() {
            let j = neighbors[ni];
            let k_idx = neighbors[(ni + 1) % neighbors.len()];
            let vj = mesh.points.get(j);
            let vk = mesh.points.get(k_idx);
            let a = [vj[0] - p[0], vj[1] - p[1], vj[2] - p[2]];
            let b = [vk[0] - p[0], vk[1] - p[1], vk[2] - p[2]];
            let la = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
            let lb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
            if la > 1e-15 && lb > 1e-15 {
                let cos_angle = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / (la * lb);
                angle_sum += cos_angle.clamp(-1.0, 1.0).acos();
            }
        }
        gauss_curv.push(2.0 * std::f64::consts::PI - angle_sum);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MeanCurvature",
            mean_curv,
            1,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GaussianCurvature",
            gauss_curv,
            1,
        )));
    result
}

/// Classify surface type by curvature signs.
/// 0=flat, 1=elliptic(dome), 2=hyperbolic(saddle), 3=parabolic(cylinder)
pub fn classify_surface_type(mesh: &PolyData) -> PolyData {
    let with_curv = curvature_analysis(mesh);
    let h_arr = with_curv.point_data().get_array("MeanCurvature").unwrap();
    let g_arr = with_curv
        .point_data()
        .get_array("GaussianCurvature")
        .unwrap();
    let n = h_arr.num_tuples();
    let mut hb = [0.0f64];
    let mut gb = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            h_arr.tuple_as_f64(i, &mut hb);
            g_arr.tuple_as_f64(i, &mut gb);
            let h = hb[0];
            let g = gb[0];
            if h.abs() < 0.01 && g.abs() < 0.01 {
                0.0
            }
            // flat
            else if g > 0.01 {
                1.0
            }
            // elliptic
            else if g < -0.01 {
                2.0
            }
            // hyperbolic
            else {
                3.0
            } // parabolic
        })
        .collect();

    let mut result = with_curv;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SurfaceType",
            data,
            1,
        )));
    result
}

fn build_adj(m: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut a: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for c in m.polys.iter() {
        let nc = c.len();
        for i in 0..nc {
            let x = c[i] as usize;
            let y = c[(i + 1) % nc] as usize;
            if x < n && y < n {
                a[x].insert(y);
                a[y].insert(x);
            }
        }
    }
    a.into_iter().map(|s| s.into_iter().collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sphere_curvature() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, -1.0],
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [0, 3, 4],
                [0, 4, 1],
                [5, 2, 1],
                [5, 3, 2],
                [5, 4, 3],
                [5, 1, 4],
            ],
        );
        let result = curvature_analysis(&mesh);
        assert!(result.point_data().get_array("MeanCurvature").is_some());
        assert!(result.point_data().get_array("GaussianCurvature").is_some());
    }
    #[test]
    fn classify() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, -1.0],
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [0, 3, 4],
                [0, 4, 1],
                [5, 2, 1],
                [5, 3, 2],
                [5, 4, 3],
                [5, 1, 4],
            ],
        );
        let result = classify_surface_type(&mesh);
        assert!(result.point_data().get_array("SurfaceType").is_some());
    }
}
