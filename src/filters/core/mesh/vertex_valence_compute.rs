//! Compute vertex valence (number of incident edges) and attach as point data.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute vertex valence and attach as "Valence" point data.
pub fn compute_valence(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut neighbors: Vec<std::collections::HashSet<usize>> =
        vec![std::collections::HashSet::new(); n];
    for cell in mesh.lines.iter() {
        add_open_cell_edges(cell, n, &mut neighbors);
    }
    for cell in mesh.polys.iter() {
        add_closed_cell_edges(cell, n, &mut neighbors);
    }
    for cell in mesh.strips.iter() {
        add_triangle_strip_edges(cell, n, &mut neighbors);
    }
    let data: Vec<f64> = (0..n).map(|i| neighbors[i].len() as f64).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Valence", data, 1)));
    result
}

/// Get valence statistics (min, max, mean, irregular count).
pub fn valence_stats(mesh: &PolyData) -> (usize, usize, f64, usize) {
    let r = compute_valence(mesh);
    let arr = r.point_data().get_array("Valence").unwrap();
    let n = arr.num_tuples();
    if n == 0 {
        return (0, 0, 0.0, 0);
    }
    let mut buf = [0.0f64];
    let vals: Vec<usize> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] as usize
        })
        .collect();
    let mn = *vals.iter().min().unwrap();
    let mx = *vals.iter().max().unwrap();
    let mean = vals.iter().sum::<usize>() as f64 / n as f64;
    let irregular = vals.iter().filter(|&&v| v != 6).count(); // 6 is regular for triangulations
    (mn, mx, mean, irregular)
}

fn add_edge(a: i64, b: i64, n: usize, neighbors: &mut [std::collections::HashSet<usize>]) {
    let (Ok(a), Ok(b)) = (usize::try_from(a), usize::try_from(b)) else {
        return;
    };
    if a >= n || b >= n || a == b {
        return;
    }
    neighbors[a].insert(b);
    neighbors[b].insert(a);
}

fn add_open_cell_edges(cell: &[i64], n: usize, neighbors: &mut [std::collections::HashSet<usize>]) {
    for edge in cell.windows(2) {
        add_edge(edge[0], edge[1], n, neighbors);
    }
}

fn add_closed_cell_edges(
    cell: &[i64],
    n: usize,
    neighbors: &mut [std::collections::HashSet<usize>],
) {
    if cell.len() < 2 {
        return;
    }
    add_open_cell_edges(cell, n, neighbors);
    add_edge(cell[cell.len() - 1], cell[0], n, neighbors);
}

fn add_triangle_strip_edges(
    cell: &[i64],
    n: usize,
    neighbors: &mut [std::collections::HashSet<usize>],
) {
    for tri in cell.windows(3) {
        add_edge(tri[0], tri[1], n, neighbors);
        add_edge(tri[1], tri[2], n, neighbors);
        add_edge(tri[2], tri[0], n, neighbors);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_valence() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = compute_valence(&mesh);
        let arr = r.point_data().get_array("Valence").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 3.0); // vertex 1 connects to 0, 2, 3
    }
    #[test]
    fn test_stats() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let (mn, mx, mean, _) = valence_stats(&mesh);
        assert_eq!(mn, 2);
        assert_eq!(mx, 2);
        assert!((mean - 2.0).abs() < 1e-10);
    }
}
