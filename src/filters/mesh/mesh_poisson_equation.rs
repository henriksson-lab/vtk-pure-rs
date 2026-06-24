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
    let mut buf = [0.0f64];
    let rhs: Vec<f64> = (0..src.num_tuples())
        .map(|i| {
            src.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
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
                *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
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
