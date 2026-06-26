//! Curvature tensor analysis: mean/Gaussian curvature, shape operator eigenvalues.

use super::curvature_simple;
use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute mean and Gaussian curvature at each vertex.
pub fn curvature_analysis(mesh: &PolyData) -> PolyData {
    curvature_simple::mean_curvature(&curvature_simple::gaussian_curvature(mesh))
}

/// Classify surface type by curvature signs.
/// 0=flat, 1=elliptic(dome), 2=hyperbolic(saddle), 3=parabolic(cylinder)
pub fn classify_surface_type(mesh: &PolyData) -> PolyData {
    let with_curv = curvature_analysis(mesh);
    let h_arr = with_curv.point_data().get_array("MeanCurvature").unwrap();
    let g_arr = with_curv
        .point_data()
        .get_array("GaussianCurvature")
        .unwrap();
    let n = h_arr.num_tuples();
    let mut hb = [0.0f64];
    let mut gb = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            h_arr.tuple_as_f64(i, &mut hb);
            g_arr.tuple_as_f64(i, &mut gb);
            let h = hb[0];
            let g = gb[0];
            if h.abs() < 0.01 && g.abs() < 0.01 {
                0.0
            }
            // flat
            else if g > 0.01 {
                1.0
            }
            // elliptic
            else if g < -0.01 {
                2.0
            }
            // hyperbolic
            else {
                3.0
            } // parabolic
        })
        .collect();

    let mut result = with_curv;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SurfaceType",
            data,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sphere_curvature() {
        let mesh = crate::filters::core::sources::sphere::sphere(
            &crate::filters::core::sources::sphere::SphereParams {
                radius: 1.0,
                ..Default::default()
            },
        );
        let result = curvature_analysis(&mesh);
        assert!(result.point_data().get_array("MeanCurvature").is_some());
        assert!(result.point_data().get_array("GaussianCurvature").is_some());
    }
    #[test]
    fn classify() {
        let mesh = crate::filters::core::sources::sphere::sphere(
            &crate::filters::core::sources::sphere::SphereParams::default(),
        );
        let result = classify_surface_type(&mesh);
        assert!(result.point_data().get_array("SurfaceType").is_some());
    }
}
