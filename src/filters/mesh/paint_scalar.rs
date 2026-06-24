//! Interactive scalar painting on mesh surfaces.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Paint a scalar value onto vertices within a sphere brush.
pub fn paint_sphere_brush(
    mesh: &PolyData,
    array_name: &str,
    center: [f64; 3],
    radius: f64,
    value: f64,
    blend: f64,
) -> PolyData {
    let n = mesh.points.len();
    let r2 = radius * radius;
    let existing = mesh.point_data().get_array(array_name);
    let mut data = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        let old = if let Some(arr) = existing {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        } else {
            0.0
        };
        let p = mesh.points.get(i);
        let d2 =
            (p[0] - center[0]).powi(2) + (p[1] - center[1]).powi(2) + (p[2] - center[2]).powi(2);
        if d2 <= r2 {
            let falloff = 1.0 - (d2 / r2).sqrt();
            data.push(old * (1.0 - blend * falloff) + value * blend * falloff);
        } else {
            data.push(old);
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)));
    result
}

/// Flood fill a scalar value from a seed vertex to connected vertices
/// where the existing value is within tolerance of the seed value.
pub fn flood_fill_scalar(
    mesh: &PolyData,
    array_name: &str,
    seed_vertex: usize,
    new_value: f64,
    tolerance: f64,
) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let mut buf = [0.0f64];
    let mut values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    if seed_vertex >= n {
        return mesh.clone();
    }

    let seed_val = values[seed_vertex];
    let adj = build_adj(mesh, n);
    let mut queue = std::collections::VecDeque::new();
    let mut visited = vec![false; n];
    queue.push_back(seed_vertex);
    visited[seed_vertex] = true;
    while let Some(v) = queue.pop_front() {
        values[v] = new_value;
        for &nb in &adj[v] {
            if !visited[nb] && (values[nb] - seed_val).abs() <= tolerance {
                visited[nb] = true;
                queue.push_back(nb);
            }
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, 1,
        )));
    result
}

fn build_adj(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn brush() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [0.5, 0.0, 0.0], [5.0, 0.0, 0.0]]);
        let result = paint_sphere_brush(&mesh, "color", [0.0, 0.0, 0.0], 1.0, 1.0, 1.0);
        let arr = result.point_data().get_array("color").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.5);
        arr.tuple_as_f64(2, &mut buf);
        assert!(buf[0] < 0.01);
    }
    #[test]
    fn flood() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![0.0; 4], 1)));
        let result = flood_fill_scalar(&mesh, "v", 0, 5.0, 0.1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 5.0);
    }
}
