//! Build the graph Laplacian matrix and export diagonal as scalar data.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub struct LaplacianMatrix {
    pub n: usize,
    pub diag: Vec<f64>,
    pub off_diag: Vec<(usize, usize, f64)>,
}

pub fn build_laplacian(mesh: &PolyData) -> LaplacianMatrix {
    let n = mesh.points.len();
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        for i in 0..cell.len() {
            add_edge(&mut adj, n, cell[i], cell[(i + 1) % cell.len()]);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_edge(&mut adj, n, edge[0], edge[1]);
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
            add_edge(&mut adj, n, tri[0], tri[1]);
            add_edge(&mut adj, n, tri[1], tri[2]);
            add_edge(&mut adj, n, tri[2], tri[0]);
        }
    }
    let diag: Vec<f64> = adj.iter().map(|s| s.len() as f64).collect();
    let mut off_diag = Vec::new();
    for (i, neighbors) in adj.iter().enumerate() {
        for &j in neighbors {
            if i < j {
                off_diag.push((i, j, -1.0));
            }
        }
    }
    LaplacianMatrix { n, diag, off_diag }
}

pub fn laplacian_diagonal(mesh: &PolyData) -> PolyData {
    let lap = build_laplacian(mesh);
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "LaplacianDiag",
            lap.diag,
            1,
        )));
    result.point_data_mut().set_active_scalars("LaplacianDiag");
    result
}

fn add_edge(adj: &mut [std::collections::HashSet<usize>], n: usize, a_id: i64, b_id: i64) {
    if a_id < 0 || b_id < 0 {
        return;
    }
    let a = a_id as usize;
    let b = b_id as usize;
    if a >= n || b >= n || a == b {
        return;
    }
    adj[a].insert(b);
    adj[b].insert(a);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_laplacian() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let lap = build_laplacian(&mesh);
        assert_eq!(lap.n, 3);
        assert_eq!(lap.diag[0], 2.0); // vertex 0 has 2 neighbors
        let r = laplacian_diagonal(&mesh);
        assert!(r.point_data().get_array("LaplacianDiag").is_some());
    }
}
