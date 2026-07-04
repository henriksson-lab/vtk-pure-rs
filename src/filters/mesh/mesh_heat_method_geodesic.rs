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
            let Some(a) = valid_point_id(cell[i], n) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], n) else {
                continue;
            };
            if a == b {
                continue;
            }
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                .sqrt();
            if !nb[a].iter().any(|&(x, _)| x == b) {
                nb[a].push((b, d));
            }
            if !nb[b].iter().any(|&(x, _)| x == a) {
                nb[b].push((a, d));
            }
        }
    }
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            if !valid_triangle(tri, n) {
                continue;
            }
            add_weighted_edge(tri[0], tri[1], mesh, &mut nb);
            add_weighted_edge(tri[1], tri[2], mesh, &mut nb);
            add_weighted_edge(tri[2], tri[0], mesh, &mut nb);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_weighted_edge(edge[0], edge[1], mesh, &mut nb);
        }
    }
    // Step 1: Diffuse heat from source
    let num_heat_steps = heat_steps.max(1);
    let dt = diffusion_time / num_heat_steps as f64;
    let mut u = vec![0.0f64; n];
    u[source] = 1.0;
    for _ in 0..num_heat_steps {
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
        let Some(ids) = valid_triangle_ids(cell, n) else {
            continue;
        };
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
    for strip in mesh.strips.iter() {
        for (i, tri) in strip.windows(3).enumerate() {
            if !valid_triangle(tri, n) {
                continue;
            }
            let tri = if i % 2 == 0 {
                [tri[0], tri[1], tri[2]]
            } else {
                [tri[1], tri[0], tri[2]]
            };
            accumulate_triangle_gradient(mesh, &tri, &u, &mut grad);
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
    for _ in 0..num_heat_steps * 2 {
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

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
}

fn valid_triangle_ids(cell: &[i64], n_points: usize) -> Option<[usize; 3]> {
    Some([
        valid_point_id(*cell.first()?, n_points)?,
        valid_point_id(*cell.get(1)?, n_points)?,
        valid_point_id(*cell.get(2)?, n_points)?,
    ])
}

fn valid_triangle(tri: &[i64], n_points: usize) -> bool {
    tri.len() == 3
        && tri[0] != tri[1]
        && tri[1] != tri[2]
        && tri[2] != tri[0]
        && tri
            .iter()
            .all(|&point_id| valid_point_id(point_id, n_points).is_some())
}

fn add_weighted_edge(a: i64, b: i64, mesh: &PolyData, nb: &mut [Vec<(usize, f64)>]) {
    let n = mesh.points.len();
    let Some(a) = valid_point_id(a, n) else {
        return;
    };
    let Some(b) = valid_point_id(b, n) else {
        return;
    };
    if a == b {
        return;
    }
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
    if !nb[a].iter().any(|&(x, _)| x == b) {
        nb[a].push((b, d));
    }
    if !nb[b].iter().any(|&(x, _)| x == a) {
        nb[b].push((a, d));
    }
}

fn accumulate_triangle_gradient(mesh: &PolyData, tri: &[i64; 3], u: &[f64], grad: &mut [[f64; 3]]) {
    let n = mesh.points.len();
    let Some(ids) = valid_triangle_ids(tri, n) else {
        return;
    };
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
        return;
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

    #[test]
    fn zero_heat_steps_still_runs_one_iteration() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = heat_method_distance(&m, 0, 0.5, 0);
        let arr = r.point_data().get_array("HeatGeodesic").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 0.1);
    }
}
