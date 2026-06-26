//! Smooth vertex normals by averaging neighbor normals.

use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashSet;

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

    let neighbors = vertex_neighbors(mesh, n);

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

fn vertex_neighbors(mesh: &PolyData, npoints: usize) -> Vec<HashSet<usize>> {
    let mut neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); npoints];

    for cell in mesh.polys.iter() {
        add_closed_cell_edges(cell, npoints, &mut neighbors);
    }
    for cell in mesh.lines.iter() {
        add_open_cell_edges(cell, npoints, &mut neighbors);
    }
    for strip in mesh.strips.iter() {
        add_triangle_strip_edges(strip, npoints, &mut neighbors);
    }

    neighbors
}

fn add_closed_cell_edges(cell: &[i64], npoints: usize, neighbors: &mut [HashSet<usize>]) {
    if cell.len() < 2 {
        return;
    }
    for i in 0..cell.len() {
        add_edge(cell[i], cell[(i + 1) % cell.len()], npoints, neighbors);
    }
}

fn add_open_cell_edges(cell: &[i64], npoints: usize, neighbors: &mut [HashSet<usize>]) {
    for edge in cell.windows(2) {
        add_edge(edge[0], edge[1], npoints, neighbors);
    }
}

fn add_triangle_strip_edges(strip: &[i64], npoints: usize, neighbors: &mut [HashSet<usize>]) {
    if strip.len() < 3 {
        return;
    }
    for i in 0..strip.len() - 2 {
        let tri = if i % 2 == 0 {
            [strip[i], strip[i + 1], strip[i + 2]]
        } else {
            [strip[i + 1], strip[i], strip[i + 2]]
        };
        add_edge(tri[0], tri[1], npoints, neighbors);
        add_edge(tri[1], tri[2], npoints, neighbors);
        add_edge(tri[2], tri[0], npoints, neighbors);
    }
}

fn add_edge(a: i64, b: i64, npoints: usize, neighbors: &mut [HashSet<usize>]) {
    if a < 0 || b < 0 {
        return;
    }
    let a = a as usize;
    let b = b as usize;
    if a >= npoints || b >= npoints || a == b {
        return;
    }
    neighbors[a].insert(b);
    neighbors[b].insert(a);
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
    fn triangle_strip_neighbors_are_decomposed() {
        let mut mesh = PolyData::new();
        for p in [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ] {
            mesh.points.push(p);
        }
        mesh.strips.push_cell(&[0, 1, 2, 3]);

        let neighbors = vertex_neighbors(&mesh, mesh.points.len());
        assert!(neighbors[0].contains(&2));
        assert!(neighbors[1].contains(&3));
        assert!(!neighbors[0].contains(&3));
    }
}
