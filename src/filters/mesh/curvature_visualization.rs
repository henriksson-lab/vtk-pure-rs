//! Curvature visualization: compute and map curvature to colors/scalars.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute discrete Gaussian curvature via angle deficit.
pub fn gaussian_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut angle_sum = vec![0.0f64; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let pts: Vec<[f64; 3]> = cell.iter().map(|&p| mesh.points.get(p as usize)).collect();
        let nc = pts.len();
        for vi in 0..nc {
            let prev = pts[(vi + nc - 1) % nc];
            let curr = pts[vi];
            let next = pts[(vi + 1) % nc];
            let e1 = [next[0] - curr[0], next[1] - curr[1], next[2] - curr[2]];
            let e2 = [prev[0] - curr[0], prev[1] - curr[1], prev[2] - curr[2]];
            let l1 = (e1[0] * e1[0] + e1[1] * e1[1] + e1[2] * e1[2]).sqrt();
            let l2 = (e2[0] * e2[0] + e2[1] * e2[1] + e2[2] * e2[2]).sqrt();
            if l1 > 1e-15 && l2 > 1e-15 {
                let cos_a = (e1[0] * e2[0] + e1[1] * e2[1] + e1[2] * e2[2]) / (l1 * l2);
                angle_sum[cell[vi] as usize] += cos_a.clamp(-1.0, 1.0).acos();
            }
        }
    }
    let curvature: Vec<f64> = angle_sum
        .iter()
        .map(|&a| 2.0 * std::f64::consts::PI - a)
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GaussianCurvature",
            curvature,
            1,
        )));
    result
}

/// Compute discrete mean curvature via Laplacian magnitude.
pub fn mean_curvature_magnitude(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let mut curv = Vec::with_capacity(n);
    for i in 0..n {
        if adj[i].is_empty() {
            curv.push(0.0);
            continue;
        }
        let p = mesh.points.get(i);
        let mut lap = [0.0; 3];
        for &j in &adj[i] {
            let q = mesh.points.get(j);
            for c in 0..3 {
                lap[c] += q[c] - p[c];
            }
        }
        let k = adj[i].len() as f64;
        curv.push(((lap[0] / k).powi(2) + (lap[1] / k).powi(2) + (lap[2] / k).powi(2)).sqrt());
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MeanCurvatureMag",
            curv,
            1,
        )));
    result
}

/// Map curvature values to RGB colors (blue→white→red diverging).
pub fn curvature_to_rgb(mesh: &PolyData, curvature_array: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(curvature_array) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut max_abs = 0.0f64;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        max_abs = max_abs.max(buf[0].abs());
    }
    if max_abs < 1e-15 {
        max_abs = 1.0;
    }

    let mut rgb = Vec::with_capacity(n * 3);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let t = (buf[0] / max_abs).clamp(-1.0, 1.0);
        if t > 0.0 {
            rgb.push(1.0);
            rgb.push(1.0 - t);
            rgb.push(1.0 - t);
        }
        // positive=red
        else {
            rgb.push(1.0 + t);
            rgb.push(1.0 + t);
            rgb.push(1.0);
        } // negative=blue
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CurvatureRGB",
            rgb,
            3,
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
    fn gaussian() {
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
        let result = gaussian_curvature(&mesh);
        assert!(result.point_data().get_array("GaussianCurvature").is_some());
    }
    #[test]
    fn mean() {
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
        let result = mean_curvature_magnitude(&mesh);
        assert!(result.point_data().get_array("MeanCurvatureMag").is_some());
    }
    #[test]
    fn to_rgb() {
        let mut mesh = PolyData::from_triangles(
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
        mesh = gaussian_curvature(&mesh);
        let result = curvature_to_rgb(&mesh, "GaussianCurvature");
        assert!(result.point_data().get_array("CurvatureRGB").is_some());
    }
}
