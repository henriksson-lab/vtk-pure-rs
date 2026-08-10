//! One-ring / N-ring vertex neighbourhood extraction.
//!
//! Both entry points are re-exported from
//! [`crate::filters::mesh::vertex_ring_ops`], which holds the single
//! implementation.

/// Extract the one-ring neighborhood of a vertex.
///
/// Returns a PolyData containing only the faces that share the given vertex,
/// plus all points referenced by those faces.
pub use crate::filters::mesh::vertex_ring_ops::extract_one_ring;

/// Extract the N-ring neighborhood of a vertex.
pub use crate::filters::mesh::vertex_ring_ops::extract_n_ring;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;

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
