//! Solve Poisson equation on mesh surface with source terms.
//!
//! The Jacobi solver itself lives in
//! [`crate::filters::mesh::laplacian_solve::poisson_solve`], which takes the
//! Dirichlet values as an explicit (vertex, value) list. This entry point keeps
//! the convenience signature: it detects the boundary (edges used by a single
//! face) and pins every boundary vertex to the same value.
use crate::data::PolyData;

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
    match mesh.point_data().get_array(source_array) {
        Some(array) if array.num_components() == 1 && array.num_tuples() >= n => {}
        _ => return mesh.clone(),
    }

    let boundary_values: Vec<(usize, f64)> = boundary_vertices(mesh, n)
        .into_iter()
        .map(|vertex| (vertex, boundary_value))
        .collect();

    let mut result = super::laplacian_solve::poisson_solve(
        mesh,
        source_array,
        &boundary_values,
        iterations,
        "Solution",
    );
    result.point_data_mut().set_active_scalars("Solution");
    result
}

/// Vertices on the mesh boundary: endpoints of an edge used by exactly one face.
fn boundary_vertices(mesh: &PolyData, n: usize) -> Vec<usize> {
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            add_counted_edge(&mut edge_count, n, cell[i], cell[(i + 1) % nc]);
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
            add_counted_edge(&mut edge_count, n, tri[0], tri[1]);
            add_counted_edge(&mut edge_count, n, tri[1], tri[2]);
            add_counted_edge(&mut edge_count, n, tri[2], tri[0]);
        }
    }

    let mut boundary = std::collections::HashSet::new();
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            boundary.insert(a);
            boundary.insert(b);
        }
    }
    boundary.into_iter().collect()
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
    use crate::data::{AnyDataArray, DataArray};

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
        assert_eq!(r.point_data().scalars().unwrap().name(), "Solution");
    }

    #[test]
    fn missing_source_array_is_a_noop() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = poisson_solve(&m, "src", 0.0, 10);
        assert!(r.point_data().get_array("Solution").is_none());
    }
}
