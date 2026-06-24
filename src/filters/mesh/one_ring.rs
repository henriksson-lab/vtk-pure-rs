use crate::data::{CellArray, Points, PolyData};
use std::collections::{HashMap, HashSet};

/// Extract the one-ring neighborhood of a vertex.
///
/// Returns a PolyData containing only the faces that share the given vertex,
/// plus all points referenced by those faces.
pub fn extract_one_ring(input: &PolyData, vertex_id: usize) -> PolyData {
    let vid = vertex_id as i64;
    let mut pt_map: HashMap<i64, i64> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.iter().any(|&id| id == vid) {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    *pt_map.entry(id).or_insert_with(|| {
                        let idx = out_points.len() as i64;
                        out_points.push(input.points.get(id as usize));
                        idx
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

/// Extract the N-ring neighborhood of a vertex.
pub fn extract_n_ring(input: &PolyData, vertex_id: usize, n: usize) -> PolyData {
    let mut current_verts: HashSet<i64> = HashSet::new();
    current_verts.insert(vertex_id as i64);

    // Build adjacency
    let mut adj: HashMap<i64, Vec<i64>> = HashMap::new();
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            adj.entry(a).or_default().push(b);
            adj.entry(b).or_default().push(a);
        }
    }

    for _ in 0..n {
        let mut next = current_verts.clone();
        for &v in &current_verts {
            if let Some(nbrs) = adj.get(&v) {
                for &nb in nbrs {
                    next.insert(nb);
                }
            }
        }
        current_verts = next;
    }

    // Extract cells that have ALL vertices in the neighborhood
    let mut pt_map: HashMap<i64, i64> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.iter().all(|&id| current_verts.contains(&id)) {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    *pt_map.entry(id).or_insert_with(|| {
                        let idx = out_points.len() as i64;
                        out_points.push(input.points.get(id as usize));
                        idx
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fan() -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // center
        for i in 0..6 {
            let a = std::f64::consts::PI * 2.0 * i as f64 / 6.0;
            pd.points.push([a.cos(), a.sin(), 0.0]);
        }
        for i in 0..6 {
            pd.polys
                .push_cell(&[0, (i + 1) as i64, ((i + 1) % 6 + 1) as i64]);
        }
        pd
    }

    #[test]
    fn one_ring_center() {
        let pd = make_fan();
        let ring = extract_one_ring(&pd, 0);
        assert_eq!(ring.polys.num_cells(), 6); // center touches all 6
    }

    #[test]
    fn one_ring_edge() {
        let pd = make_fan();
        let ring = extract_one_ring(&pd, 1);
        assert_eq!(ring.polys.num_cells(), 2); // vertex 1 in 2 triangles
    }

    #[test]
    fn n_ring_grows() {
        let pd = make_fan();
        let r1 = extract_n_ring(&pd, 1, 1);
        let r2 = extract_n_ring(&pd, 1, 2);
        assert!(r2.polys.num_cells() >= r1.polys.num_cells());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let ring = extract_one_ring(&pd, 0);
        assert_eq!(ring.polys.num_cells(), 0);
    }
}
