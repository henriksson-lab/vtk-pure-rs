//! Classify vertices by curvature type (elliptic, hyperbolic, parabolic, flat).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn classify_curvature(mesh: &PolyData) -> PolyData {
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
                let cos =
                    ((va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2]) / (la * lb)).clamp(-1.0, 1.0);
                angle_sum[vi] += cos.acos();
            }
        }
    }
    let gauss: Vec<f64> = angle_sum
        .iter()
        .map(|&s| 2.0 * std::f64::consts::PI - s)
        .collect();
    // Classify: >eps=elliptic(1), <-eps=hyperbolic(-1), ~0=parabolic/flat(0)
    let eps = 0.01;
    let data: Vec<f64> = gauss
        .iter()
        .map(|&g| {
            if g > eps {
                1.0
            } else if g < -eps {
                -1.0
            } else {
                0.0
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CurvatureType",
            data,
            1,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GaussianCurv",
            gauss,
            1,
        )));
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
                [0.5, 0.3, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = classify_curvature(&m);
        assert!(r.point_data().get_array("CurvatureType").is_some());
        assert!(r.point_data().get_array("GaussianCurv").is_some());
    }
}
