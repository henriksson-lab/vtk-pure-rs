//! Consistent face orientation.
//!
//! The single implementation lives in [`crate::filters::mesh::mesh_orient`];
//! this module re-exports it so the `orient_faces::orient_faces_consistent`
//! path keeps working.

pub use crate::filters::mesh::mesh_orient::orient_faces_consistent;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = orient_faces_consistent(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn propagates_from_flipped_face() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([2.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 2, 0]); // must flip, then drives the next face
        pd.polys.push_cell(&[2, 3, 4]);

        let result = orient_faces_consistent(&pd);
        let cells: Vec<Vec<i64>> = result.polys.iter().map(|c| c.to_vec()).collect();
        assert_eq!(cells[1], vec![0, 2, 3]);
        assert_eq!(cells[2], vec![4, 3, 2]);
    }

    #[test]
    fn visits_disconnected_components() {
        let mut pd = PolyData::new();
        for p in [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [3.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 1.0, 0.0],
            [3.0, 1.0, 0.0],
        ] {
            pd.points.push(p);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 2, 0]);
        pd.polys.push_cell(&[4, 5, 6]);
        pd.polys.push_cell(&[7, 6, 4]);

        let result = orient_faces_consistent(&pd);
        let cells: Vec<Vec<i64>> = result.polys.iter().map(|c| c.to_vec()).collect();
        assert_eq!(cells[1], vec![0, 2, 3]);
        assert_eq!(cells[3], vec![4, 6, 7]);
    }
}
