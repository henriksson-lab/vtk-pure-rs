//! Discrete Ricci flow on mesh (conformal deformation toward constant curvature).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn discrete_ricci_flow(mesh: &PolyData, steps: usize, dt: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
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
    // Conformal factor u (initially 0, so metric = e^{2u} * flat)
    let mut u = vec![0.0f64; n];
    // Compute target curvature (2pi*chi/n for each vertex)
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
                angle_sum[vi] += ((va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2]) / (la * lb))
                    .clamp(-1.0, 1.0)
                    .acos();
            }
        }
    }
    let gauss_curv: Vec<f64> = angle_sum
        .iter()
        .map(|&s| 2.0 * std::f64::consts::PI - s)
        .collect();
    let total_curv: f64 = gauss_curv.iter().sum();
    let target = total_curv / n as f64;
    // Flow: du/dt = target - K_i (evolve conformal factor)
    for _ in 0..steps {
        for i in 0..n {
            u[i] += dt * (target - gauss_curv[i]);
        }
    }
    // Apply conformal scaling to mesh positions
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        cx += pos[i][0];
        cy += pos[i][1];
        cz += pos[i][2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;
    for i in 0..n {
        let scale = (u[i] * 0.1).exp(); // damped
        pos[i] = [
            (pos[i][0] - cx) * scale + cx,
            (pos[i][1] - cy) * scale + cy,
            (pos[i][2] - cz) * scale + cz,
        ];
    }
    let mut r = mesh.clone();
    for i in 0..n {
        r.points.set(i, pos[i]);
    }
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ConformalFactor",
            u,
            1,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GaussCurvature",
            gauss_curv,
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
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = discrete_ricci_flow(&m, 10, 0.01);
        assert!(r.point_data().get_array("ConformalFactor").is_some());
        assert!(r.point_data().get_array("GaussCurvature").is_some());
    }
}
