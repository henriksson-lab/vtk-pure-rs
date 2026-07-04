//! Compute vertex valence (degree) and store as scalar data.
use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::{BTreeMap, HashSet};

pub fn valence_stats(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let seen = build_vertex_neighbors(mesh, n);
    let data: Vec<f64> = seen.iter().map(|s| s.len() as f64).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Valence", data, 1)));
    result.point_data_mut().set_active_scalars("Valence");
    result
}

pub fn valence_histogram(mesh: &PolyData) -> Vec<(u32, usize)> {
    let n = mesh.points.len();
    let seen = build_vertex_neighbors(mesh, n);
    let mut hist = BTreeMap::new();
    for v in seen.iter().map(|s| s.len() as u32) {
        *hist.entry(v).or_insert(0usize) += 1;
    }
    hist.into_iter().collect()
}

fn build_vertex_neighbors(mesh: &PolyData, n: usize) -> Vec<HashSet<usize>> {
    let mut seen: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    add_poly_edges(&mesh.polys, n, &mut seen);
    add_triangle_strip_edges(&mesh.strips, n, &mut seen);
    add_line_edges(&mesh.lines, n, &mut seen);
    seen
}

fn add_poly_edges(cells: &crate::data::CellArray, n: usize, seen: &mut [HashSet<usize>]) {
    for cell in cells.iter() {
        if cell.len() < 2 || !cell.iter().all(|&id| id >= 0 && (id as usize) < n) {
            continue;
        }
        for i in 0..cell.len() {
            add_edge(cell[i] as usize, cell[(i + 1) % cell.len()] as usize, seen);
        }
    }
}

fn add_triangle_strip_edges(cells: &crate::data::CellArray, n: usize, seen: &mut [HashSet<usize>]) {
    for cell in cells.iter() {
        if cell.len() < 3 || !cell.iter().all(|&id| id >= 0 && (id as usize) < n) {
            continue;
        }
        for tri in cell.windows(3) {
            add_edge(tri[0] as usize, tri[1] as usize, seen);
            add_edge(tri[1] as usize, tri[2] as usize, seen);
            add_edge(tri[2] as usize, tri[0] as usize, seen);
        }
    }
}

fn add_line_edges(cells: &crate::data::CellArray, n: usize, seen: &mut [HashSet<usize>]) {
    for cell in cells.iter() {
        if cell.len() < 2 || !cell.iter().all(|&id| id >= 0 && (id as usize) < n) {
            continue;
        }
        for edge in cell.windows(2) {
            add_edge(edge[0] as usize, edge[1] as usize, seen);
        }
    }
}

fn add_edge(a: usize, b: usize, seen: &mut [HashSet<usize>]) {
    if a != b {
        seen[a].insert(b);
        seen[b].insert(a);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_valence() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = valence_stats(&mesh);
        let arr = r.point_data().get_array("Valence").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 2.0); // each vertex has 2 neighbors in single triangle
    }
}
