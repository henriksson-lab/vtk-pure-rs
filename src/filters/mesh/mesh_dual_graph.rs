//! Construct dual graph: each face becomes a vertex, connected faces share edges.

pub use crate::filters::mesh::dual_graph::dual_graph;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;

    #[test]
    fn non_adjacent_faces_are_not_linked() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [3.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [3.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = dual_graph(&mesh);
        assert_eq!(r.points.len(), 2);
        assert_eq!(r.lines.num_cells(), 0);
    }
}
