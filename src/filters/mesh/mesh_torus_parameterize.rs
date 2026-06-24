//! Parameterize mesh vertices on a torus (for torus-like meshes).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn torus_uv_parameterize(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;
    let mut uvs = Vec::with_capacity(n * 2);
    for i in 0..n {
        let p = mesh.points.get(i);
        let dx = p[0] - cx;
        let dy = p[1] - cy;
        let u = (dy.atan2(dx) / (2.0 * std::f64::consts::PI) + 0.5).rem_euclid(1.0);
        let rxy = (dx * dx + dy * dy).sqrt();
        let major_r = if n > 10 {
            let dists: Vec<f64> = (0..n)
                .map(|j| {
                    let q = mesh.points.get(j);
                    ((q[0] - cx).powi(2) + (q[1] - cy).powi(2)).sqrt()
                })
                .collect();
            dists.iter().sum::<f64>() / nf
        } else {
            rxy
        };
        let dz = p[2] - cz;
        let dr = rxy - major_r;
        let v = (dz.atan2(dr) / (2.0 * std::f64::consts::PI) + 0.5).rem_euclid(1.0);
        uvs.push(u);
        uvs.push(v);
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("UV", uvs, 2)));
    r.point_data_mut().set_active_tcoords("UV");
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [3.0, 0.0, 0.0],
                [0.0, 3.0, 0.0],
                [-3.0, 0.0, 0.0],
                [0.0, -3.0, 0.0],
            ],
            vec![[0, 1, 2], [2, 3, 0]],
        );
        let r = torus_uv_parameterize(&m);
        assert!(r.point_data().get_array("UV").is_some());
        assert_eq!(r.point_data().get_array("UV").unwrap().num_components(), 2);
    }
}
