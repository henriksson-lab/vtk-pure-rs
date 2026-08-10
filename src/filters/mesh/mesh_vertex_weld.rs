//! Weld vertices closer than tolerance.
//!
//! The single implementation lives in [`crate::filters::mesh::weld_vertices`];
//! this module re-exports it so the `mesh_vertex_weld::weld_vertices` path
//! keeps working.

pub use crate::filters::mesh::weld_vertices::weld_vertices;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;

    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.001, 0.001, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = weld_vertices(&m, 0.01);
        assert!(r.points.len() <= 3);
    }
}
