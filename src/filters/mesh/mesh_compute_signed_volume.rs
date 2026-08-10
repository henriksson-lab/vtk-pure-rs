//! Compute signed and unsigned volume of closed meshes.
use crate::data::PolyData;

pub use crate::filters::mesh::volume::compactness;
pub use crate::filters::mesh::volume::signed_volume;
pub use crate::filters::mesh::volume::surface_area;

pub fn unsigned_volume(mesh: &PolyData) -> f64 {
    signed_volume(mesh).abs()
}
pub fn is_volume_positive(mesh: &PolyData) -> bool {
    signed_volume(mesh) > 0.0
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vol() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let v = unsigned_volume(&m);
        assert!((v - 1.0 / 6.0).abs() < 0.05);
    }
    #[test]
    fn winding_flip_flips_the_volume_sign() {
        let outward = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let inward = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 3, 1], [1, 3, 2], [0, 2, 3]],
        );
        assert_ne!(is_volume_positive(&outward), is_volume_positive(&inward));
        assert!((unsigned_volume(&outward) - unsigned_volume(&inward)).abs() < 1e-12);
    }
}
