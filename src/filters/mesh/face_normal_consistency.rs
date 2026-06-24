use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

/// Check face normal consistency and mark inconsistent edges.
///
/// Two adjacent triangles are consistent if their shared edge has
/// opposite directions in each triangle. Adds "NormalConsistency"
/// cell data (1.0 = all edges consistent, 0.0 = has inconsistent edge).
pub fn check_normal_consistency(input: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();

    // Directed edges per face
    let mut directed: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            directed.entry((a, b)).or_default().push(fi);
        }
    }

    let mut consistency = vec![1.0f64; nc];

    // An edge (a,b) should appear at most once as directed edge.
    // If (a,b) appears in two faces, they're inconsistent (should be (a,b) and (b,a)).
    for ((a, b), faces) in &directed {
        if faces.len() > 1 {
            // Same directed edge in multiple faces = inconsistent
            for &fi in faces {
                consistency[fi] = 0.0;
            }
        }
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalConsistency",
            consistency,
            1,
        )));
    pd
}

/// Count the number of inconsistent edges.
pub fn count_inconsistent_edges(input: &PolyData) -> usize {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let mut directed: HashMap<(i64, i64), usize> = HashMap::new();
    for c in &cells {
        for i in 0..c.len() {
            *directed.entry((c[i], c[(i + 1) % c.len()])).or_insert(0) += 1;
        }
    }
    directed.values().filter(|&&c| c > 1).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consistent_pair() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]); // edge 0->1, 1->2, 2->0
        pd.polys.push_cell(&[0, 2, 3]); // edge 0->2 (reversed: 2->0 in first) OK

        let result = check_normal_consistency(&pd);
        let arr = result.cell_data().get_array("NormalConsistency").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        assert_eq!(count_inconsistent_edges(&pd), 0);
    }

    #[test]
    fn inconsistent_pair() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 1, 3]); // edge 2->1 same as 1->2 reversed? No: 1->2 in first, but here we have 2->1 = correct

        // Actually make them truly inconsistent: both go 0->1
        let mut pd2 = PolyData::new();
        pd2.points.push([0.0, 0.0, 0.0]);
        pd2.points.push([1.0, 0.0, 0.0]);
        pd2.points.push([1.0, 1.0, 0.0]);
        pd2.points.push([0.0, 1.0, 0.0]);
        pd2.polys.push_cell(&[0, 1, 2]);
        pd2.polys.push_cell(&[0, 1, 3]); // edge 0->1 appears twice!

        assert!(count_inconsistent_edges(&pd2) > 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(count_inconsistent_edges(&pd), 0);
    }
}
