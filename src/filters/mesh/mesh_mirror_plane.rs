//! Mirror mesh across an arbitrary plane.

pub use super::mesh_mirror::mirror_plane;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;

    #[test]
    fn test_mirror() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = mirror_plane(&mesh, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let p = r.points.get(0);
        assert!((p[0] - (-1.0)).abs() < 1e-9); // reflected across YZ plane
    }
}
