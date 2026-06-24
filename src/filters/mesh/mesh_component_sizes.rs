//! Label each vertex with the size of its connected component.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn component_sizes(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
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
    let mut comp_id = vec![-1i32; n];
    let mut comp_sizes: Vec<usize> = Vec::new();
    let mut id = 0i32;
    for start in 0..n {
        if comp_id[start] >= 0 {
            continue;
        }
        let mut size = 0usize;
        let mut stack = vec![start];
        comp_id[start] = id;
        while let Some(v) = stack.pop() {
            size += 1;
            for &nb in &adj[v] {
                if comp_id[nb] < 0 {
                    comp_id[nb] = id;
                    stack.push(nb);
                }
            }
        }
        comp_sizes.push(size);
        id += 1;
    }
    let sizes: Vec<f64> = comp_id
        .iter()
        .map(|&c| {
            if c >= 0 {
                comp_sizes[c as usize] as f64
            } else {
                0.0
            }
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ComponentSize",
            sizes,
            1,
        )));
    result.point_data_mut().set_active_scalars("ComponentSize");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_comp_sizes() {
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
        let r = component_sizes(&mesh);
        let arr = r.point_data().get_array("ComponentSize").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 3.0); // first component has 3 vertices
    }
}
