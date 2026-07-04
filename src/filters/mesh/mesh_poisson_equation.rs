//! Solve Poisson equation on mesh surface with source terms.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn poisson_solve(
    mesh: &PolyData,
    source_array: &str,
    boundary_value: f64,
    iterations: usize,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let src = match mesh.point_data().get_array(source_array) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    if src.num_tuples() < n {
        return mesh.clone();
    }
    let mut buf = [0.0f64];
    let rhs: Vec<f64> = (0..n)
        .map(|i| {
            src.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            add_neighbor_pair(&mut nb, n, cell[i], cell[(i + 1) % nc]);
            add_counted_edge(&mut ec, n, cell[i], cell[(i + 1) % nc]);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_neighbor_pair(&mut nb, n, edge[0], edge[1]);
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
            add_neighbor_pair(&mut nb, n, tri[0], tri[1]);
            add_neighbor_pair(&mut nb, n, tri[1], tri[2]);
            add_neighbor_pair(&mut nb, n, tri[2], tri[0]);
            add_counted_edge(&mut ec, n, tri[0], tri[1]);
            add_counted_edge(&mut ec, n, tri[1], tri[2]);
            add_counted_edge(&mut ec, n, tri[2], tri[0]);
        }
    }
    let mut boundary = std::collections::HashSet::new();
    for (&(a, b), &c) in &ec {
        if c == 1 {
            boundary.insert(a);
            boundary.insert(b);
        }
    }
    let mut u = vec![0.0f64; n];
    for &bi in &boundary {
        u[bi] = boundary_value;
    }
    for _ in 0..iterations {
        let prev = u.clone();
        for i in 0..n {
            if boundary.contains(&i) || nb[i].is_empty() {
                continue;
            }
            let k = nb[i].len() as f64;
            u[i] = (nb[i].iter().map(|&j| prev[j]).sum::<f64>() + rhs[i]) / k;
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Solution", u, 1)));
    r.point_data_mut().set_active_scalars("Solution");
    r
}

fn add_neighbor_pair(nb: &mut [Vec<usize>], n: usize, a_id: i64, b_id: i64) {
    let Some(a) = valid_point_id(a_id, n) else {
        return;
    };
    let Some(b) = valid_point_id(b_id, n) else {
        return;
    };
    if a == b {
        return;
    }
    if !nb[a].contains(&b) {
        nb[a].push(b);
    }
    if !nb[b].contains(&a) {
        nb[b].push(a);
    }
}

fn add_counted_edge(
    edge_count: &mut std::collections::HashMap<(usize, usize), usize>,
    n: usize,
    a_id: i64,
    b_id: i64,
) {
    let Some(a) = valid_point_id(a_id, n) else {
        return;
    };
    let Some(b) = valid_point_id(b_id, n) else {
        return;
    };
    if a == b {
        return;
    }
    *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
}

fn valid_point_id(point_id: i64, npoints: usize) -> Option<usize> {
    usize::try_from(point_id).ok().filter(|&id| id < npoints)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "src",
                vec![0.0, 0.0, 1.0, 0.0],
                1,
            )));
        let r = poisson_solve(&m, "src", 0.0, 50);
        assert!(r.point_data().get_array("Solution").is_some());
    }
}
