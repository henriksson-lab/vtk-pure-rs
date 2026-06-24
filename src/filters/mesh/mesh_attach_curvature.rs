//! Attach discrete curvature estimates as point data.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn attach_mean_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
            }
        }
    }
    let data: Vec<f64> = (0..n)
        .map(|i| {
            if nb[i].is_empty() {
                return 0.0;
            }
            let p = mesh.points.get(i);
            let mut lap = [0.0, 0.0, 0.0];
            for &j in &nb[i] {
                let q = mesh.points.get(j);
                lap[0] += q[0] - p[0];
                lap[1] += q[1] - p[1];
                lap[2] += q[2] - p[2];
            }
            let k = nb[i].len() as f64;
            lap[0] /= k;
            lap[1] /= k;
            lap[2] /= k;
            (lap[0] * lap[0] + lap[1] * lap[1] + lap[2] * lap[2]).sqrt()
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MeanCurvature",
            data,
            1,
        )));
    r.point_data_mut().set_active_scalars("MeanCurvature");
    r
}
pub fn attach_gaussian_curvature(mesh: &PolyData) -> PolyData {
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
    let data: Vec<f64> = angle_sum
        .iter()
        .map(|&s| 2.0 * std::f64::consts::PI - s)
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GaussianCurvature",
            data,
            1,
        )));
    r.point_data_mut().set_active_scalars("GaussianCurvature");
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mean() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = attach_mean_curvature(&m);
        assert!(r.point_data().get_array("MeanCurvature").is_some());
    }
    #[test]
    fn test_gauss() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_gaussian_curvature(&m);
        assert!(r.point_data().get_array("GaussianCurvature").is_some());
    }
}
