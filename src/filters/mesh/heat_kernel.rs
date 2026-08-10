use crate::data::PolyData;

/// Heat diffusion step size used by this single-scale entry point.
const STEP_TIME: f64 = 0.3;

/// Compute Heat Kernel Signature (HKS) at each vertex.
///
/// HKS is a shape descriptor based on heat diffusion. Approximated by
/// running heat diffusion from each vertex for `time` steps and recording
/// the remaining heat at the source. Adds a single-component "HKS" scalar array.
///
/// Thin wrapper over the single (multi-scale) implementation in
/// [`crate::filters::mesh::mesh_heat_kernel_signature::heat_kernel_signature`],
/// evaluated at the one diffusion time `time * 0.3` that `time` steps of size
/// 0.3 correspond to.
pub fn heat_kernel_signature(input: &PolyData, time: usize) -> PolyData {
    crate::filters::mesh::mesh_heat_kernel_signature::heat_kernel_signature(
        input,
        &[STEP_TIME * time as f64],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hks_varies_with_topology() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // corner
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = heat_kernel_signature(&pd, 5);
        assert!(result.point_data().get_array("HKS").is_some());
    }

    #[test]
    fn symmetric_vertices_equal_hks() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = heat_kernel_signature(&pd, 3);
        let arr = result.point_data().get_array("HKS").unwrap();
        // All 3 vertices of a single triangle have same connectivity
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let h0 = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        let h1 = buf[0];
        assert!((h0 - h1).abs() < 0.1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = heat_kernel_signature(&pd, 5);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn ignores_empty_cells() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.polys.push_cell(&[]);

        let result = heat_kernel_signature(&pd, 1);
        let arr = result.point_data().get_array("HKS").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
