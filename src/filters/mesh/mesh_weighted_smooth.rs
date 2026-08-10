//! Weighted Laplacian smoothing with per-vertex weights.
//!
//! Thin wrapper around
//! [`crate::filters::mesh::laplacian_smooth_weighted::weighted_laplacian_smooth`],
//! which holds the single implementation; only the argument order differs.
use crate::data::PolyData;

/// Smooth with per-vertex weights from a scalar array.
///
/// `lambda` is the base relaxation factor, scaled per vertex by the (clamped)
/// weight array.
pub fn weighted_laplacian_smooth(
    mesh: &PolyData,
    weight_array: &str,
    iterations: usize,
    lambda: f64,
) -> PolyData {
    crate::filters::mesh::laplacian_smooth_weighted::weighted_laplacian_smooth(
        mesh,
        weight_array,
        lambda,
        iterations,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "w",
                vec![0.0, 0.0, 0.0, 1.0],
                1,
            )));
        let r = weighted_laplacian_smooth(&m, "w", 5, 0.5);
        // Vertices 0,1,2 should not move (weight=0), vertex 3 should smooth
        let p0 = r.points.get(0);
        assert!((p0[0]).abs() < 1e-10);
    }
}
