use crate::data::PolyData;
use std::collections::HashSet;

/// Count the number of duplicate (overlapping) faces in a PolyData.
///
/// Two faces are duplicates if they share the same set of vertex indices,
/// regardless of ordering.
pub fn count_duplicate_faces(input: &PolyData) -> usize {
    let mut seen = HashSet::new();
    let mut duplicates: usize = 0;

    for cell in input.polys.iter() {
        let mut sorted: Vec<i64> = cell.to_vec();
        sorted.sort();
        if !seen.insert(sorted) {
            duplicates += 1;
        }
    }

    duplicates
}

/// Remove duplicate faces from a PolyData, keeping only the first occurrence.
///
/// Two faces are considered duplicates if they share the same set of vertex
/// indices (order-independent).
pub fn remove_duplicate_faces(input: &PolyData) -> PolyData {
    let mut seen = HashSet::new();
    let mut output = PolyData::new();
    output.points = input.points.clone();

    for cell in input.polys.iter() {
        let mut sorted: Vec<i64> = cell.to_vec();
        sorted.sort();
        if seen.insert(sorted) {
            output.polys.push_cell(cell);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_duplicates() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        assert_eq!(count_duplicate_faces(&pd), 0);
        let result = remove_duplicate_faces(&pd);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn exact_duplicate() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 2]); // exact duplicate

        assert_eq!(count_duplicate_faces(&pd), 1);
        let result = remove_duplicate_faces(&pd);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn reordered_duplicate() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 0, 1]); // same vertices, different order

        assert_eq!(count_duplicate_faces(&pd), 1);
        let result = remove_duplicate_faces(&pd);
        assert_eq!(result.polys.num_cells(), 1);
    }
}
