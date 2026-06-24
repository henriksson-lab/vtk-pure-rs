//! Compute cotangent Laplacian weights and store mean curvature.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn cotan_mean_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    let mut cotan_lap = vec![[0.0f64; 3]; n];
    let mut area = vec![0.0f64; n];
    for &[a, b, c] in &tris {
        if a >= n || b >= n || c >= n {
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let edges = [
            [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]],
            [pc[0] - pb[0], pc[1] - pb[1], pc[2] - pb[2]],
            [pa[0] - pc[0], pa[1] - pc[1], pa[2] - pc[2]],
        ];
        let cot = |u: &[f64; 3], v: &[f64; 3]| -> f64 {
            let dot = u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
            let cross = ((u[1] * v[2] - u[2] * v[1]).powi(2)
                + (u[2] * v[0] - u[0] * v[2]).powi(2)
                + (u[0] * v[1] - u[1] * v[0]).powi(2))
            .sqrt();
            if cross > 1e-15 {
                dot / cross
            } else {
                0.0
            }
        };
        // Cotangent at each vertex
        let cot_a = cot(&edges[0], &[-edges[2][0], -edges[2][1], -edges[2][2]]);
        let cot_b = cot(&edges[1], &[-edges[0][0], -edges[0][1], -edges[0][2]]);
        let cot_c = cot(&edges[2], &[-edges[1][0], -edges[1][1], -edges[1][2]]);
        // Cotan Laplacian contribution
        for d in 0..3 {
            cotan_lap[a][d] += cot_c * (pb[d] - pa[d]) + cot_b * (pc[d] - pa[d]);
            cotan_lap[b][d] += cot_a * (pc[d] - pb[d]) + cot_c * (pa[d] - pb[d]);
            cotan_lap[c][d] += cot_b * (pa[d] - pc[d]) + cot_a * (pb[d] - pc[d]);
        }
        // Voronoi area
        let tri_area = 0.5
            * ((edges[0][1] * edges[2][2] - edges[0][2] * edges[2][1]).powi(2)
                + (edges[0][2] * edges[2][0] - edges[0][0] * edges[2][2]).powi(2)
                + (edges[0][0] * edges[2][1] - edges[0][1] * edges[2][0]).powi(2))
            .sqrt();
        area[a] += tri_area / 3.0;
        area[b] += tri_area / 3.0;
        area[c] += tri_area / 3.0;
    }
    let mean_curv: Vec<f64> = (0..n)
        .map(|i| {
            if area[i] > 1e-15 {
                let h = 0.5
                    * (cotan_lap[i][0] * cotan_lap[i][0]
                        + cotan_lap[i][1] * cotan_lap[i][1]
                        + cotan_lap[i][2] * cotan_lap[i][2])
                        .sqrt()
                    / area[i];
                h
            } else {
                0.0
            }
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CotanMeanCurvature",
            mean_curv,
            1,
        )));
    result
        .point_data_mut()
        .set_active_scalars("CotanMeanCurvature");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cotan() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = cotan_mean_curvature(&mesh);
        assert!(r.point_data().get_array("CotanMeanCurvature").is_some());
    }
}
