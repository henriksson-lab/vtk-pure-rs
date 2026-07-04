//! Smooth vertex normals by averaging neighbor normals.

use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};

/// Smooth vertex normals by averaging with neighbor normals over N iterations.
pub fn smooth_vertex_normals(mesh: &PolyData, iterations: usize) -> PolyData {
    let normals_arr = match mesh
        .point_data()
        .normals()
        .or_else(|| mesh.point_data().get_array("Normals"))
    {
        Some(a) if a.num_components() == 3 => a,
        _ => return mesh.clone(),
    };

    let n = mesh.points.len();
    if normals_arr.num_tuples() < n {
        return mesh.clone();
    }

    let mut buf = [0.0f64; 3];
    let mut normals: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            normals_arr.tuple_as_f64(i, &mut buf);
            [buf[0], buf[1], buf[2]]
        })
        .collect();

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    add_closed_cell_edges(&mesh.polys, n, &mut neighbors);
    add_open_cell_edges(&mesh.lines, n, &mut neighbors);
    add_triangle_strip_edges(&mesh.strips, n, &mut neighbors);

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
    result.point_data_mut().set_active_normals("Normals");
    result
}

fn add_closed_cell_edges(cells: &CellArray, npoints: usize, neighbors: &mut [Vec<usize>]) {
    for cell in cells.iter() {
        let Some(indices) = valid_cell_indices(cell, npoints) else {
            continue;
        };
        if indices.len() < 2 {
            continue;
        }
        for i in 0..indices.len() {
            add_edge(neighbors, indices[i], indices[(i + 1) % indices.len()]);
        }
    }
}

fn add_open_cell_edges(cells: &CellArray, npoints: usize, neighbors: &mut [Vec<usize>]) {
    for cell in cells.iter() {
        let Some(indices) = valid_cell_indices(cell, npoints) else {
            continue;
        };
        for edge in indices.windows(2) {
            add_edge(neighbors, edge[0], edge[1]);
        }
    }
}

fn add_triangle_strip_edges(cells: &CellArray, npoints: usize, neighbors: &mut [Vec<usize>]) {
    for cell in cells.iter() {
        let Some(indices) = valid_cell_indices(cell, npoints) else {
            continue;
        };
        if indices.len() < 3 {
            continue;
        }
        for i in 0..indices.len() - 2 {
            let tri = if i % 2 == 0 {
                [indices[i], indices[i + 1], indices[i + 2]]
            } else {
                [indices[i + 1], indices[i], indices[i + 2]]
            };
            add_edge(neighbors, tri[0], tri[1]);
            add_edge(neighbors, tri[1], tri[2]);
            add_edge(neighbors, tri[2], tri[0]);
        }
    }
}

fn add_edge(neighbors: &mut [Vec<usize>], a: usize, b: usize) {
    if a == b {
        return;
    }
    if !neighbors[a].contains(&b) {
        neighbors[a].push(b);
    }
    if !neighbors[b].contains(&a) {
        neighbors[b].push(a);
    }
}

fn valid_cell_indices(cell: &[i64], npoints: usize) -> Option<Vec<usize>> {
    let mut indices = Vec::with_capacity(cell.len());
    for &id in cell {
        if id < 0 || id as usize >= npoints {
            return None;
        }
        indices.push(id as usize);
    }
    Some(indices)
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
        assert!(result.point_data().normals().is_some());
    }

    #[test]
    fn smooths_active_normals_array() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.lines.push_cell(&[0, 1]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "n",
                vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                3,
            )));
        mesh.point_data_mut().set_active_normals("n");

        let result = smooth_vertex_normals(&mesh, 1);
        let arr = result.point_data().get_array("Normals").unwrap();
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        let inv_sqrt_2 = 1.0 / 2.0f64.sqrt();
        assert!((buf[0] - inv_sqrt_2).abs() < 1e-10);
        assert!((buf[1] - inv_sqrt_2).abs() < 1e-10);
        assert!(result.point_data().normals().is_some());
    }

    #[test]
    fn triangle_strip_does_not_smooth_across_closing_edge() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.strips.push_cell(&[0, 1, 2, 3]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
                3,
            )));

        let result = smooth_vertex_normals(&mesh, 1);
        let arr = result.point_data().get_array("Normals").unwrap();
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[2] > 0.99, "normal = {:?}", buf);
    }

    #[test]
    fn verts_do_not_create_smoothing_neighbors() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.verts.push_cell(&[0, 1]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                3,
            )));

        let result = smooth_vertex_normals(&mesh, 1);
        let arr = result.point_data().get_array("Normals").unwrap();
        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [1.0, 0.0, 0.0]);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [0.0, 1.0, 0.0]);
    }
}
