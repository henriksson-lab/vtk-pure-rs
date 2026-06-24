use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashSet;

/// Compute vertex valence (number of adjacent edges) for each point.
///
/// Adds a "Valence" scalar point data array. For regular triangle meshes,
/// interior vertices have valence 6 and boundary vertices have lower.
pub fn vertex_valence(input: &PolyData) -> PolyData {
    let n = input.points.len();
    let mut neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); n];

    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            neighbors[a].insert(b);
            neighbors[b].insert(a);
        }
    }

    let valences: Vec<f64> = neighbors.iter().map(|s| s.len() as f64).collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Valence", valences, 1,
        )));
    pd
}

/// Compute valence histogram: returns (valence, count) pairs.
pub fn valence_histogram(input: &PolyData) -> Vec<(usize, usize)> {
    let n = input.points.len();
    let mut neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); n];

    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            neighbors[a].insert(b);
            neighbors[b].insert(a);
        }
    }

    let mut counts = std::collections::HashMap::new();
    for s in &neighbors {
        *counts.entry(s.len()).or_insert(0usize) += 1;
    }

    let mut result: Vec<(usize, usize)> = counts.into_iter().collect();
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_valence() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = vertex_valence(&pd);
        let arr = result.point_data().get_array("Valence").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 2.0); // each vertex connected to 2 others
        }
    }

    #[test]
    fn fan_valence() {
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

        let result = vertex_valence(&pd);
        let arr = result.point_data().get_array("Valence").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 6.0); // center has valence 6
    }

    #[test]
    fn histogram() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let hist = valence_histogram(&pd);
        assert!(!hist.is_empty());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = vertex_valence(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
