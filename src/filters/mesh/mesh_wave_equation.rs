//! Wave equation simulation on mesh surface.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn wave_simulate(
    mesh: &PolyData,
    initial: &[f64],
    velocity: f64,
    time: f64,
    steps: usize,
) -> PolyData {
    let n = mesh.points.len();
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            add_edge(cell[i], cell[(i + 1) % nc], &mut nb);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_edge(edge[0], edge[1], &mut nb);
        }
    }
    for strip in mesh.strips.iter() {
        for (i, tri) in strip.windows(3).enumerate() {
            if i % 2 == 0 {
                add_triangle_edges(tri[0], tri[1], tri[2], &mut nb);
            } else {
                add_triangle_edges(tri[1], tri[0], tri[2], &mut nb);
            }
        }
    }
    let dt = time / steps.max(1) as f64;
    let c2 = velocity * velocity;
    let mut u: Vec<f64> = if initial.len() >= n {
        initial[..n].to_vec()
    } else {
        let mut v = initial.to_vec();
        v.resize(n, 0.0);
        v
    };
    let mut u_prev = u.clone();
    for _ in 0..steps {
        let mut u_next = vec![0.0f64; n];
        for i in 0..n {
            if nb[i].is_empty() {
                u_next[i] = u[i];
                continue;
            }
            let k = nb[i].len() as f64;
            let lap: f64 = nb[i].iter().map(|&j| u[j] - u[i]).sum::<f64>() / k;
            u_next[i] = 2.0 * u[i] - u_prev[i] + c2 * dt * dt * lap;
        }
        u_prev = u;
        u = u_next;
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Wave", u, 1)));
    r.point_data_mut().set_active_scalars("Wave");
    r
}
pub fn wave_from_point(
    mesh: &PolyData,
    source: usize,
    velocity: f64,
    time: f64,
    steps: usize,
) -> PolyData {
    let n = mesh.points.len();
    let mut initial = vec![0.0f64; n];
    if source < n {
        initial[source] = 1.0;
    }
    wave_simulate(mesh, &initial, velocity, time, steps)
}

fn add_edge(a: i64, b: i64, nb: &mut [Vec<usize>]) {
    let Some(a) = valid_point_id(a, nb.len()) else {
        return;
    };
    let Some(b) = valid_point_id(b, nb.len()) else {
        return;
    };
    if !nb[a].contains(&b) {
        nb[a].push(b);
    }
    if !nb[b].contains(&a) {
        nb[b].push(a);
    }
}

fn add_triangle_edges(a: i64, b: i64, c: i64, nb: &mut [Vec<usize>]) {
    add_edge(a, b, nb);
    add_edge(b, c, nb);
    add_edge(c, a, nb);
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
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
        let r = wave_from_point(&m, 0, 1.0, 0.5, 20);
        assert!(r.point_data().get_array("Wave").is_some());
    }
}
