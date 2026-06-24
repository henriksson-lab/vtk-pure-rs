//! Heat method for fast geodesic distance computation.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn heat_method_distance(
    mesh: &PolyData,
    source: usize,
    diffusion_time: f64,
    heat_steps: usize,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || source >= n {
        return mesh.clone();
    }
    let mut nb: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                let d =
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt();
                if !nb[a].iter().any(|&(x, _)| x == b) {
                    nb[a].push((b, d));
                    nb[b].push((a, d));
                }
            }
        }
    }
    // Step 1: Diffuse heat from source
    let dt = diffusion_time / heat_steps.max(1) as f64;
    let mut u = vec![0.0f64; n];
    u[source] = 1.0;
    for _ in 0..heat_steps {
        let prev = u.clone();
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let k = nb[i].len() as f64;
            u[i] = prev[i] + dt * nb[i].iter().map(|&(j, _)| prev[j] - prev[i]).sum::<f64>() / k;
        }
    }
    // Step 2: Compute normalized gradient direction
    let mut grad = vec![[0.0f64; 3]; n];
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
        let e1 = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
        let e2 = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
        let nm = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let a2 = nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2];
        if a2 < 1e-30 {
            continue;
        }
        let du1 = u[ids[1]] - u[ids[0]];
        let du2 = u[ids[2]] - u[ids[0]];
        let g = [
            du1 * (nm[1] * e2[2] - nm[2] * e2[1]) / a2 - du2 * (nm[1] * e1[2] - nm[2] * e1[1]) / a2,
            du1 * (nm[2] * e2[0] - nm[0] * e2[2]) / a2 - du2 * (nm[2] * e1[0] - nm[0] * e1[2]) / a2,
            du1 * (nm[0] * e2[1] - nm[1] * e2[0]) / a2 - du2 * (nm[0] * e1[1] - nm[1] * e1[0]) / a2,
        ];
        let gl = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt().max(1e-15);
        for &vi in &ids {
            grad[vi][0] -= g[0] / gl;
            grad[vi][1] -= g[1] / gl;
            grad[vi][2] -= g[2] / gl;
        }
    }
    // Normalize gradients
    for g in &mut grad {
        let l = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
        if l > 1e-15 {
            g[0] /= l;
            g[1] /= l;
            g[2] /= l;
        }
    }
    // Step 3: Solve Poisson equation for distance (Jacobi iteration)
    let mut dist = vec![0.0f64; n];
    let nb_uniform: Vec<Vec<usize>> = nb
        .iter()
        .map(|v| v.iter().map(|&(j, _)| j).collect())
        .collect();
    for _ in 0..heat_steps * 2 {
        let prev = dist.clone();
        for i in 0..n {
            if i == source || nb_uniform[i].is_empty() {
                continue;
            }
            let k = nb_uniform[i].len() as f64;
            let lap: f64 = nb_uniform[i]
                .iter()
                .map(|&j| prev[j] - prev[i])
                .sum::<f64>()
                / k;
            // Divergence of normalized gradient as RHS
            let div: f64 = nb_uniform[i]
                .iter()
                .map(|&j| {
                    let p = mesh.points.get(j);
                    let pi = mesh.points.get(i);
                    let e = [p[0] - pi[0], p[1] - pi[1], p[2] - pi[2]];
                    let el = (e[0] * e[0] + e[1] * e[1] + e[2] * e[2]).sqrt().max(1e-15);
                    (grad[j][0] * e[0] + grad[j][1] * e[1] + grad[j][2] * e[2]) / el
                })
                .sum::<f64>()
                / k;
            dist[i] = prev[i] + 0.5 * (lap - div);
        }
    }
    // Shift so source=0
    let src_d = dist[source];
    for d in &mut dist {
        *d -= src_d;
        *d = d.abs();
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "HeatGeodesic",
            dist,
            1,
        )));
    r.point_data_mut().set_active_scalars("HeatGeodesic");
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
        let r = heat_method_distance(&m, 0, 0.5, 20);
        let arr = r.point_data().get_array("HeatGeodesic").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 0.1);
    }
}
