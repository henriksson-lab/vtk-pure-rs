//! Shape Diameter Function (SDF) for thickness estimation.
//!
//! The single implementation lives in [`crate::filters::mesh::shape_diameter`];
//! this module is a thin wrapper that keeps the historical signature (an
//! explicit RNG `seed`) and the `"SDF"` output array name.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the shape diameter function and attach it as an `"SDF"` scalar.
///
/// `seed` is retained for API compatibility only: the surviving implementation
/// uses a deterministic golden-angle cone sampling pattern, so results no
/// longer depend on a random seed.
pub fn shape_diameter_function(mesh: &PolyData, num_rays: usize, _seed: u64) -> PolyData {
    let computed = crate::filters::mesh::shape_diameter::shape_diameter_function(mesh, num_rays);
    let Some(array) = computed.point_data().get_array("ShapeDiameter") else {
        return mesh.clone();
    };
    let sdf = array.to_f64_vec();

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("SDF", sdf, 1)));
    result.point_data_mut().set_active_scalars("SDF");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = shape_diameter_function(&m, 3, 42);
        assert!(r.point_data().get_array("SDF").is_some());
    }
}
