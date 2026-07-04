//! Count and label connected components of a mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn connected_components(mesh: &PolyData) -> (usize, PolyData) {
    let n = mesh.points.len();
    if n == 0 {
        return (0, mesh.clone());
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cells in [&mesh.verts, &mesh.lines, &mesh.polys, &mesh.strips] {
        add_cell_adjacencies(cells, n, &mut adj);
    }
    let mut labels = vec![-1i32; n];
    let mut component = 0i32;
    for start in 0..n {
        if labels[start] >= 0 {
            continue;
        }
        labels[start] = component;
        let mut stack = vec![start];
        while let Some(v) = stack.pop() {
            for &nb in &adj[v] {
                if labels[nb] < 0 {
                    labels[nb] = component;
                    stack.push(nb);
                }
            }
        }
        component += 1;
    }
    let label_data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Component",
            label_data,
            1,
        )));
    result.point_data_mut().set_active_scalars("Component");
    (component as usize, result)
}

fn add_cell_adjacencies(cells: &crate::data::CellArray, n: usize, adj: &mut [Vec<usize>]) {
    for cell in cells.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], n) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], n) else {
                continue;
            };
            add_edge(adj, a, b);
        }
    }
}

fn add_edge(adj: &mut [Vec<usize>], a: usize, b: usize) {
    if !adj[a].contains(&b) {
        adj[a].push(b);
    }
    if !adj[b].contains(&a) {
        adj[b].push(a);
    }
}

fn valid_point_id(id: i64, n: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < n)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_components() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [5.0, 5.0, 0.0],
                [6.0, 5.0, 0.0],
                [5.5, 6.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let (nc, r) = connected_components(&mesh);
        assert_eq!(nc, 2);
        assert!(r.point_data().get_array("Component").is_some());
    }
}
