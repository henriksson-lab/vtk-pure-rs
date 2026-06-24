//! Smooth vertex normals by averaging neighbor normals.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Smooth vertex normals by averaging with neighbor normals over N iterations.
pub fn smooth_vertex_normals(mesh: &PolyData, iterations: usize) -> PolyData {
    let normals_arr = match mesh.point_data().get_array("Normals") {
        Some(a) if a.num_components() == 3 => a,
        _ => return mesh.clone(),
    };

    let n = mesh.points.len();
    let mut buf = [0.0f64; 3];
    let mut normals: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            normals_arr.tuple_as_f64(i, &mut buf);
            [buf[0], buf[1], buf[2]]
        })
        .collect();

    // Build adjacency
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                neighbors[a].push(b);
                neighbors[b].push(a);
            }
        }
    }

    for _ in 0..iterations {
        let mut new_normals = normals.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            let mut avg = normals[i];
            for &nb in &neighbors[i] {
                avg[0] += normals[nb][0];
                avg[1] += normals[nb][1];
                avg[2] += normals[nb][2];
            }
            let len = (avg[0] * avg[0] + avg[1] * avg[1] + avg[2] * avg[2]).sqrt();
            if len > 1e-15 {
                new_normals[i] = [avg[0] / len, avg[1] / len, avg[2] / len];
            }
        }
        normals = new_normals;
    }

    let data: Vec<f64> = normals.iter().flat_map(|n| n.iter().copied()).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Normals", data, 3)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_smooth_normals() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        // Add normals pointing up
        let ndata: Vec<f64> = (0..4).flat_map(|_| vec![0.0, 0.0, 1.0]).collect();
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("Normals", ndata, 3)));
        let result = smooth_vertex_normals(&mesh, 3);
        let arr = result.point_data().get_array("Normals").unwrap();
        assert_eq!(arr.num_tuples(), 4);
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[2] - 1.0).abs() < 1e-10); // still pointing up
    }
}
