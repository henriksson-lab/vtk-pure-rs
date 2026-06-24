//! Laplace-Beltrami curvature operator on mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn laplace_beltrami_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut lap = vec![[0.0f64; 3]; n];
    let mut area = vec![0.0f64; n];
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
            let cross_l = ((eij[1] * eik[2] - eij[2] * eik[1]).powi(2)
                + (eij[2] * eik[0] - eij[0] * eik[2]).powi(2)
                + (eij[0] * eik[1] - eij[1] * eik[0]).powi(2))
            .sqrt();
            let cot = if cross_l > 1e-15 { dot / cross_l } else { 0.0 };
            let ejk = [p[k][0] - p[j][0], p[k][1] - p[j][1], p[k][2] - p[j][2]];
            for d in 0..3 {
                lap[ids[j]][d] += cot * ejk[d] * 0.5;
                lap[ids[k]][d] -= cot * ejk[d] * 0.5;
            }
        }
        let tri_area = 0.5
            * ((p[1][0] - p[0][0]) * (p[2][1] - p[0][1])
                - (p[1][1] - p[0][1]) * (p[2][0] - p[0][0]))
                .abs();
        for &id in &ids {
            area[id] += tri_area / 3.0;
        }
    }
    let mean_curv: Vec<f64> = (0..n)
        .map(|i| {
            if area[i] > 1e-15 {
                (lap[i][0] * lap[i][0] + lap[i][1] * lap[i][1] + lap[i][2] * lap[i][2]).sqrt()
                    / area[i]
                    * 0.5
            } else {
                0.0
            }
        })
        .collect();
    let lb_vec: Vec<f64> = (0..n)
        .flat_map(|i| {
            if area[i] > 1e-15 {
                vec![
                    lap[i][0] / area[i],
                    lap[i][1] / area[i],
                    lap[i][2] / area[i],
                ]
            } else {
                vec![0.0, 0.0, 0.0]
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "LBMeanCurvature",
            mean_curv,
            1,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "LBVector", lb_vec, 3,
        )));
    r.point_data_mut().set_active_scalars("LBMeanCurvature");
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = laplace_beltrami_curvature(&m);
        assert!(r.point_data().get_array("LBMeanCurvature").is_some());
        assert!(r.point_data().get_array("LBVector").is_some());
    }
}
