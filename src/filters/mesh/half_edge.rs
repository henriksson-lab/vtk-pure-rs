use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

/// Compute half-edge valence: number of outgoing directed edges per vertex.
///
/// Differs from vertex valence when mesh has boundary or inconsistent winding.
/// Adds "HalfEdgeValence" scalar array.
pub fn half_edge_valence(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }
    let mut valence = vec![0.0f64; n];

    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            valence[cell[i] as usize] += 1.0; // one outgoing half-edge per face vertex
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "HalfEdgeValence",
            valence,
            1,
        )));
    pd
}

/// Detect non-manifold vertices: vertices where the one-ring is not a single fan.
///
/// A vertex is non-manifold if it has more incoming than outgoing edges from
/// separate fans. Adds "IsNonManifoldVertex" binary scalar.
pub fn detect_non_manifold_vertices(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut edge_count: HashMap<(i64, i64), usize> = HashMap::new();
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    // A vertex is non-manifold if any of its edges is shared by >2 faces
    let mut is_nm = vec![0.0f64; n];
    for (&(a, b), &c) in &edge_count {
        if c > 2 {
            is_nm[a as usize] = 1.0;
            is_nm[b as usize] = 1.0;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "IsNonManifoldVertex",
            is_nm,
            1,
        )));
    pd
}

/// Count non-manifold vertices.
pub fn count_non_manifold_vertices(input: &PolyData) -> usize {
    let result = detect_non_manifold_vertices(input);
    let arr = match result.point_data().get_array("IsNonManifoldVertex") {
        Some(a) => a,
        None => return 0,
    };
    let mut buf = [0.0f64];
    (0..arr.num_tuples())
        .filter(|&i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] > 0.5
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_edge_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = half_edge_valence(&pd);
        let arr = result.point_data().get_array("HalfEdgeValence").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 1.0);
        }
    }

    #[test]
    fn manifold_mesh() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        assert_eq!(count_non_manifold_vertices(&pd), 0);
    }

    #[test]
    fn non_manifold_detected() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, -1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[0, 1, 4]);

        assert!(count_non_manifold_vertices(&pd) > 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(count_non_manifold_vertices(&pd), 0);
    }
}
