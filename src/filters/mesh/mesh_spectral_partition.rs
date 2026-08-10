//! Spectral mesh partitioning using the Fiedler vector (2nd smallest eigenvector of Laplacian).
use crate::data::PolyData;

/// Partition a mesh into two halves by the sign of the Fiedler vector.
///
/// Thin wrapper around
/// [`crate::filters::mesh::laplacian_eigenmaps::spectral_partition_iter`],
/// which holds the single implementation.
pub fn spectral_partition(mesh: &PolyData, iterations: usize) -> PolyData {
    crate::filters::mesh::laplacian_eigenmaps::spectral_partition_iter(mesh, iterations)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_partition() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
                [3.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4], [3, 5, 4]],
        );
        let r = spectral_partition(&mesh, 100);
        assert!(r.point_data().get_array("Partition").is_some());
    }
}
