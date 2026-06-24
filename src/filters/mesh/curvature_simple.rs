//! Simple discrete curvature estimation.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute discrete Gaussian curvature using angle defect.
pub fn gaussian_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut angle_sum = vec![0.0f64; n];

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let nc = cell.len();
        for i in 0..nc {
            let vi = cell[i] as usize;
            let prev = cell[(i + nc - 1) % nc] as usize;
            let next = cell[(i + 1) % nc] as usize;
            let p = mesh.points.get(vi);
            let a = mesh.points.get(prev);
            let b = mesh.points.get(next);
            let va = [a[0] - p[0], a[1] - p[1], a[2] - p[2]];
            let vb = [b[0] - p[0], b[1] - p[1], b[2] - p[2]];
            let la = (va[0] * va[0] + va[1] * va[1] + va[2] * va[2]).sqrt();
            let lb = (vb[0] * vb[0] + vb[1] * vb[1] + vb[2] * vb[2]).sqrt();
            if la > 1e-15 && lb > 1e-15 {
                let cos_a = (va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2]) / (la * lb);
                angle_sum[vi] += cos_a.clamp(-1.0, 1.0).acos();
            }
        }
    }

    let data: Vec<f64> = angle_sum
        .iter()
        .map(|&s| 2.0 * std::f64::consts::PI - s)
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GaussianCurvature",
            data,
            1,
        )));
    result
        .point_data_mut()
        .set_active_scalars("GaussianCurvature");
    result
}

/// Compute discrete mean curvature using cotangent Laplacian magnitude.
pub fn mean_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut laplacian = vec![[0.0f64; 3]; n];
    let mut area = vec![0.0f64; n];

    for cell in mesh.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let ids = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
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
            let cross_len = cross_mag(eij, eik);
            let cot = if cross_len > 1e-15 {
                dot / cross_len
            } else {
                0.0
            };
            let ejk = [p[k][0] - p[j][0], p[k][1] - p[j][1], p[k][2] - p[j][2]];
            laplacian[ids[j]][0] += cot * ejk[0] * 0.5;
            laplacian[ids[j]][1] += cot * ejk[1] * 0.5;
            laplacian[ids[j]][2] += cot * ejk[2] * 0.5;
            laplacian[ids[k]][0] -= cot * ejk[0] * 0.5;
            laplacian[ids[k]][1] -= cot * ejk[1] * 0.5;
            laplacian[ids[k]][2] -= cot * ejk[2] * 0.5;
        }
        let tri_area = 0.5
            * cross_mag(
                [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]],
                [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]],
            );
        for &id in &ids {
            area[id] += tri_area / 3.0;
        }
    }

    let data: Vec<f64> = (0..n)
        .map(|i| {
            if area[i] > 1e-15 {
                let l = &laplacian[i];
                (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt() / area[i] * 0.5
            } else {
                0.0
            }
        })
        .collect();

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MeanCurvature",
            data,
            1,
        )));
    result.point_data_mut().set_active_scalars("MeanCurvature");
    result
}

fn cross_mag(a: [f64; 3], b: [f64; 3]) -> f64 {
    let c = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gaussian() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = gaussian_curvature(&mesh);
        assert!(r.point_data().get_array("GaussianCurvature").is_some());
    }
    #[test]
    fn test_mean() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = mean_curvature(&mesh);
        assert!(r.point_data().get_array("MeanCurvature").is_some());
    }
}
