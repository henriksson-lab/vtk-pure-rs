//! Mesh saliency based on multi-scale mean curvature differences.
use crate::data::{AnyDataArray, DataArray, PolyData};

/// Multi-scale mesh saliency (Lee et al. 2005).
///
/// For every scale sigma the per-scale saliency `|G(C, sigma) - G(C, 2*sigma)|`
/// is accumulated. An empty `scales` slice falls back to `[1, 2, 4]`.
///
/// The single-scale kernel is implemented once in
/// [`crate::filters::mesh::saliency::mesh_saliency`]; this function only
/// accumulates it across scales.
pub fn mesh_saliency(mesh: &PolyData, scales: &[f64]) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    let used_scales = if scales.is_empty() {
        vec![1.0, 2.0, 4.0]
    } else {
        scales.to_vec()
    };

    let mut saliency = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for &sigma in &used_scales {
        let per_scale = crate::filters::mesh::saliency::mesh_saliency(mesh, sigma, 2.0 * sigma);
        let Some(arr) = per_scale.point_data().get_array("Saliency") else {
            continue;
        };
        for (i, value) in saliency.iter_mut().enumerate().take(arr.num_tuples()) {
            arr.tuple_as_f64(i, &mut buf);
            *value += buf[0];
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Saliency", saliency, 1,
        )));
    result.point_data_mut().set_active_scalars("Saliency");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_saliency() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = mesh_saliency(&mesh, &[1.0, 2.0]);
        assert!(r.point_data().get_array("Saliency").is_some());
    }
}
