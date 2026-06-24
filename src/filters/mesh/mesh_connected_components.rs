//! Count and label connected components of a mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn connected_components(mesh: &PolyData) -> (usize, PolyData) {
    let n = mesh.points.len();
    if n == 0 {
        return (0, mesh.clone());
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !adj[a].contains(&b) {
                    adj[a].push(b);
                }
                if !adj[b].contains(&a) {
                    adj[b].push(a);
                }
            }
        }
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
