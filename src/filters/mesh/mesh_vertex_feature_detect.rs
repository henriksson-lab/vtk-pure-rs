//! Detect feature vertices (corners, edges, flat regions).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn detect_feature_vertices(mesh: &PolyData, angle_threshold: f64) -> PolyData {
    let n = mesh.points.len();
    let cos_t = angle_threshold.to_radians().cos();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut vf: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (ci, c) in cells.iter().enumerate() {
        for &v in c {
            vf[v as usize].push(ci);
        }
    }
    let fnormals: Vec<[f64; 3]> = cells
        .iter()
        .map(|c| {
            if c.len() < 3 {
                return [0.0, 0.0, 1.0];
            }
            let a = mesh.points.get(c[0] as usize);
            let b = mesh.points.get(c[1] as usize);
            let cc = mesh.points.get(c[2] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [cc[0] - a[0], cc[1] - a[1], cc[2] - a[2]];
            let nn = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let l = (nn[0] * nn[0] + nn[1] * nn[1] + nn[2] * nn[2]).sqrt();
            if l < 1e-15 {
                [0.0, 0.0, 1.0]
            } else {
                [nn[0] / l, nn[1] / l, nn[2] / l]
            }
        })
        .collect();
    // Feature type: 0=flat, 1=edge, 2=corner
    let data: Vec<f64> = (0..n)
        .map(|i| {
            if vf[i].len() < 2 {
                return 0.0;
            }
            let mut sharp_count = 0;
            for fi in 0..vf[i].len() {
                for fj in fi + 1..vf[i].len() {
                    let n1 = fnormals[vf[i][fi]];
                    let n2 = fnormals[vf[i][fj]];
                    let dot = n1[0] * n2[0] + n1[1] * n2[1] + n1[2] * n2[2];
                    if dot < cos_t {
                        sharp_count += 1;
                    }
                }
            }
            if sharp_count >= 2 {
                2.0
            } else if sharp_count >= 1 {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FeatureType",
            data,
            1,
        )));
    r.point_data_mut().set_active_scalars("FeatureType");
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
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let r = detect_feature_vertices(&m, 30.0);
        assert!(r.point_data().get_array("FeatureType").is_some());
    }
}
