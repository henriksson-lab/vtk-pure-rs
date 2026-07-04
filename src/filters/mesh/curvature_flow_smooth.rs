//! Curvature flow smoothing: move vertices along their normal proportional
//! to mean curvature. This flow minimizes surface area.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Mean curvature flow smoothing.
///
/// Each vertex moves along its estimated normal proportional to the
/// discrete Laplacian (approximating mean curvature). This produces
/// minimal surfaces.
pub fn mean_curvature_flow(mesh: &PolyData, iterations: usize, dt: f64) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }
    let adj = build_adj(mesh, n);
    let mut positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    for _ in 0..iterations {
        let mut new_pos = positions.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let mut lap = [0.0; 3];
            for &j in &adj[i] {
                for c in 0..3 {
                    lap[c] += positions[j][c] - positions[i][c];
                }
            }
            let k = adj[i].len() as f64;
            for c in 0..3 {
                new_pos[i][c] += dt * lap[c] / k;
            }
        }
        positions = new_pos;
    }

    let mut result = mesh.clone();
    result.points = Points::from(positions);
    result
}

/// Cotangent-weighted mean curvature flow (more accurate).
pub fn cotangent_curvature_flow(mesh: &PolyData, iterations: usize, dt: f64) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut positions: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    for _ in 0..iterations {
        let mut displacements = vec![[0.0; 3]; n];
        let mut weights = vec![0.0; n];

        // Compute symmetric cotangent Laplacian contributions per edge.
        for cell in &all_cells {
            if cell.len() != 3 {
                continue;
            }
            let vi = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
            if vi.iter().any(|&id| id >= n) {
                continue;
            }

            for &(a, b, k) in &[
                (vi[0], vi[1], vi[2]),
                (vi[1], vi[2], vi[0]),
                (vi[2], vi[0], vi[1]),
            ] {
                let pa = positions[a];
                let pb = positions[b];
                let pk = positions[k];

                let vka = [pa[0] - pk[0], pa[1] - pk[1], pa[2] - pk[2]];
                let vkb = [pb[0] - pk[0], pb[1] - pk[1], pb[2] - pk[2]];

                let dot_k = vka[0] * vkb[0] + vka[1] * vkb[1] + vka[2] * vkb[2];
                let cross_k = [
                    vka[1] * vkb[2] - vka[2] * vkb[1],
                    vka[2] * vkb[0] - vka[0] * vkb[2],
                    vka[0] * vkb[1] - vka[1] * vkb[0],
                ];
                let sin_k =
                    (cross_k[0] * cross_k[0] + cross_k[1] * cross_k[1] + cross_k[2] * cross_k[2])
                        .sqrt();
                let cot_k = if sin_k > 1e-15 { dot_k / sin_k } else { 0.0 };

                let w = cot_k.max(0.0) * 0.5; // clamp negative weights
                for c in 0..3 {
                    let edge = pb[c] - pa[c];
                    displacements[a][c] += w * edge;
                    displacements[b][c] -= w * edge;
                }
                weights[a] += w;
                weights[b] += w;
            }
        }

        let mut new_pos = positions.clone();
        for i in 0..n {
            if weights[i] <= 1e-15 {
                continue;
            }
            for c in 0..3 {
                new_pos[i][c] += dt * displacements[i][c] / weights[i];
            }
        }

        positions = new_pos;
    }

    let mut result = mesh.clone();
    result.points = Points::from(positions);
    result
}

/// Compute per-vertex mean curvature magnitude.
pub fn compute_mean_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let mut curvature = Vec::with_capacity(n);

    for i in 0..n {
        if adj[i].is_empty() {
            curvature.push(0.0);
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
        curvature.push(((lap[0] / k).powi(2) + (lap[1] / k).powi(2) + (lap[2] / k).powi(2)).sqrt());
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MeanCurvature",
            curvature,
            1,
        )));
    result
}

fn build_adj(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_bumpy() -> PolyData {
        let mut pts = Vec::new();
        for y in 0..8 {
            for x in 0..8 {
                let z = if x == 4 && y == 4 { 1.0 } else { 0.0 };
                pts.push([x as f64, y as f64, z]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..7 {
            for x in 0..7 {
                let bl = y * 8 + x;
                tris.push([bl, bl + 1, bl + 9]);
                tris.push([bl, bl + 9, bl + 8]);
            }
        }
        PolyData::from_triangles(pts, tris)
    }
    #[test]
    fn mean_flow() {
        let mesh = make_bumpy();
        let result = mean_curvature_flow(&mesh, 10, 0.1);
        let z = result.points.get(4 * 8 + 4)[2];
        assert!(z < 1.0, "bump should be smoothed");
    }
    #[test]
    fn cotangent_flow() {
        let mesh = make_bumpy();
        let result = cotangent_curvature_flow(&mesh, 5, 0.05);
        assert_eq!(result.points.len(), mesh.points.len());
    }

    #[test]
    fn cotangent_flow_is_independent_of_triangle_winding() {
        let points = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let a = PolyData::from_triangles(points.clone(), vec![[0, 1, 2]]);
        let b = PolyData::from_triangles(points, vec![[0, 2, 1]]);

        let ra = cotangent_curvature_flow(&a, 1, 0.1);
        let rb = cotangent_curvature_flow(&b, 1, 0.1);

        for i in 0..3 {
            let pa = ra.points.get(i);
            let pb = rb.points.get(i);
            for c in 0..3 {
                assert!((pa[c] - pb[c]).abs() < 1e-12);
            }
        }
    }
    #[test]
    fn curvature_computation() {
        let mesh = make_bumpy();
        let result = compute_mean_curvature(&mesh);
        let arr = result.point_data().get_array("MeanCurvature").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(4 * 8 + 4, &mut buf);
        assert!(buf[0] > 0.0); // bump has high curvature
    }
}
