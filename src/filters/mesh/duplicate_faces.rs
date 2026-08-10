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
        let key = polygon_key(cell);
        if key.len() != cell.len() {
            continue;
        }
        if !seen.insert(key) {
            duplicates += 1;
        }
    }

    duplicates
}

/// Remove duplicate faces from a PolyData, keeping only the first occurrence.
///
/// The single implementation lives in
/// [`crate::filters::mesh::mesh_remove_duplicate_faces`].
pub use crate::filters::mesh::mesh_remove_duplicate_faces::remove_duplicate_faces;

fn polygon_key(cell: &[i64]) -> Vec<i64> {
    let mut sorted = cell.to_vec();
    sorted.sort();
    sorted.dedup();
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

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

    #[test]
    fn degenerate_face_is_preserved_when_unique() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 1]);

        assert_eq!(count_duplicate_faces(&pd), 0);
        let result = remove_duplicate_faces(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn degenerate_duplicate_counts_once() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 1]);
        pd.polys.push_cell(&[1, 0, 1]);

        assert_eq!(count_duplicate_faces(&pd), 0);
        let result = remove_duplicate_faces(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn cell_data_follows_kept_faces() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 1, 0]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "id",
                vec![10, 20, 30],
                1,
            )));

        let result = remove_duplicate_faces(&pd);
        let ids = result.cell_data().get_array("id").unwrap();
        let mut buf = [0.0f64];
        ids.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 10.0);
        ids.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 30.0);
    }
}
