//! Compute total surface area and signed volume of a closed mesh.

pub use crate::filters::mesh::volume::signed_volume;
pub use crate::filters::mesh::volume::surface_area;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;
    #[test]
    fn test_area() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!((surface_area(&mesh) - 0.5).abs() < 1e-9);
    }
    #[test]
    fn test_volume() {
        // Tetrahedron: V = 1/6 for unit
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 3, 1], [0, 2, 3], [1, 3, 2]],
        );
        let v = signed_volume(&mesh).abs();
        assert!((v - 1.0 / 6.0).abs() < 0.02);
    }
}
