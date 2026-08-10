//! Shape DNA: Laplacian eigenvalue sequence as shape descriptor.
use crate::data::{AnyDataArray, DataArray, PolyData};

/// Laplacian eigenvalue sequence as a shape descriptor.
///
/// Re-exported from [`crate::filters::mesh::mesh_spectral_shape_descriptor`],
/// which holds the single implementation.
pub use crate::filters::mesh::mesh_spectral_shape_descriptor::shape_dna;

pub fn shape_dna_as_data(mesh: &PolyData, n_eigenvalues: usize, power_iters: usize) -> PolyData {
    let eigs = shape_dna(mesh, n_eigenvalues, power_iters);
    let n = mesh.points.len();
    let mut result = mesh.clone();
    if !eigs.is_empty() {
        let data = vec![eigs[0]; n]; // store first eigenvalue as scalar
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "ShapeDNA_0",
                data,
                1,
            )));
        result.point_data_mut().set_active_scalars("ShapeDNA_0");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_dna_as_data() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = shape_dna_as_data(&mesh, 3, 50);
        let arr = result.point_data().get_array("ShapeDNA_0").unwrap();
        assert_eq!(arr.num_tuples(), 4);
    }
}
