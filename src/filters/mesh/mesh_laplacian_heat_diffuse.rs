//! Heat diffusion on mesh surface using Laplacian.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn heat_diffuse(mesh: &PolyData, initial: &[f64], time: f64, steps: usize) -> PolyData {
    let n = mesh.points.len();
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    build_neighbors(mesh, &mut nb, n);
    let dt = time / steps.max(1) as f64;
    let mut u: Vec<f64> = if initial.len() >= n {
        initial[..n].to_vec()
    } else {
        let mut v = initial.to_vec();
        v.resize(n, 0.0);
        v
    };
    for _ in 0..steps {
        let prev = u.clone();
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let k = nb[i].len() as f64;
            let lap: f64 = nb[i].iter().map(|&j| prev[j] - prev[i]).sum::<f64>() / k;
            u[i] = prev[i] + dt * lap;
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Heat", u, 1)));
    r.point_data_mut().set_active_scalars("Heat");
    r
}
pub fn heat_from_vertex(mesh: &PolyData, source: usize, time: f64, steps: usize) -> PolyData {
    let n = mesh.points.len();
    let mut initial = vec![0.0f64; n];
    if source < n {
        initial[source] = 1.0;
    }
    heat_diffuse(mesh, &initial, time, steps)
}
pub fn heat_from_boundary(mesh: &PolyData, time: f64, steps: usize) -> PolyData {
    let n = mesh.points.len();
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    build_edge_counts(mesh, &mut ec, n);
    let mut initial = vec![0.0f64; n];
    for (&(a, b), &c) in &ec {
        if c == 1 {
            initial[a] = 1.0;
            initial[b] = 1.0;
        }
    }
    heat_diffuse(mesh, &initial, time, steps)
}

fn build_neighbors(mesh: &PolyData, nb: &mut [Vec<usize>], n: usize) {
    for cell in mesh.polys.iter() {
        for i in 0..cell.len() {
            add_neighbor_pair(nb, n, cell[i], cell[(i + 1) % cell.len()]);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_neighbor_pair(nb, n, edge[0], edge[1]);
        }
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            add_neighbor_pair(nb, n, tri[0], tri[1]);
            add_neighbor_pair(nb, n, tri[1], tri[2]);
            add_neighbor_pair(nb, n, tri[2], tri[0]);
        }
    }
}

fn build_edge_counts(
    mesh: &PolyData,
    ec: &mut std::collections::HashMap<(usize, usize), usize>,
    n: usize,
) {
    for cell in mesh.polys.iter() {
        for i in 0..cell.len() {
            add_edge_count(ec, n, cell[i], cell[(i + 1) % cell.len()]);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_edge_count(ec, n, edge[0], edge[1]);
        }
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            add_edge_count(ec, n, tri[0], tri[1]);
            add_edge_count(ec, n, tri[1], tri[2]);
            add_edge_count(ec, n, tri[2], tri[0]);
        }
    }
}

fn add_neighbor_pair(nb: &mut [Vec<usize>], n: usize, a_id: i64, b_id: i64) {
    if a_id < 0 || b_id < 0 {
        return;
    }
    let a = a_id as usize;
    let b = b_id as usize;
    if a >= n || b >= n || a == b {
        return;
    }
    if !nb[a].contains(&b) {
        nb[a].push(b);
    }
    if !nb[b].contains(&a) {
        nb[b].push(a);
    }
}

fn add_edge_count(
    ec: &mut std::collections::HashMap<(usize, usize), usize>,
    n: usize,
    a_id: i64,
    b_id: i64,
) {
    if a_id < 0 || b_id < 0 {
        return;
    }
    let a = a_id as usize;
    let b = b_id as usize;
    if a >= n || b >= n || a == b {
        return;
    }
    *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vertex() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = heat_from_vertex(&m, 0, 1.0, 20);
        let arr = r.point_data().get_array("Heat").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.0);
    }
    #[test]
    fn test_boundary() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = heat_from_boundary(&m, 0.5, 10);
        assert!(r.point_data().get_array("Heat").is_some());
    }
}
