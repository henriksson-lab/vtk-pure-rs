//! Curvature visualization: compute and map curvature to colors/scalars.

use super::curvature_simple;
use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute discrete Gaussian curvature via angle deficit.
pub fn gaussian_curvature(mesh: &PolyData) -> PolyData {
    curvature_simple::gaussian_curvature(mesh)
}

/// Compute discrete mean curvature via Laplacian magnitude.
pub fn mean_curvature_magnitude(mesh: &PolyData) -> PolyData {
    let with_mean = curvature_simple::mean_curvature(mesh);
    let Some(arr) = with_mean.point_data().get_array("MeanCurvature") else {
        return mesh.clone();
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut curv = Vec::with_capacity(n);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        curv.push(buf[0].abs());
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MeanCurvatureMag",
            curv,
            1,
        )));
    result
}

/// Map curvature values to RGB colors (blue→white→red diverging).
pub fn curvature_to_rgb(mesh: &PolyData, curvature_array: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(curvature_array) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut max_abs = 0.0f64;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        max_abs = max_abs.max(buf[0].abs());
    }
    if max_abs < 1e-15 {
        max_abs = 1.0;
    }

    let mut rgb = Vec::with_capacity(n * 3);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let t = (buf[0] / max_abs).clamp(-1.0, 1.0);
        if t > 0.0 {
            rgb.push(1.0);
            rgb.push(1.0 - t);
            rgb.push(1.0 - t);
        }
        // positive=red
        else {
            rgb.push(1.0 + t);
            rgb.push(1.0 + t);
            rgb.push(1.0);
        } // negative=blue
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CurvatureRGB",
            rgb,
            3,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gaussian() {
        let mesh = crate::filters::core::sources::sphere::sphere(
            &crate::filters::core::sources::sphere::SphereParams::default(),
        );
        let result = gaussian_curvature(&mesh);
        assert!(result.point_data().get_array("GaussianCurvature").is_some());
    }
    #[test]
    fn mean() {
        let mesh = crate::filters::core::sources::sphere::sphere(
            &crate::filters::core::sources::sphere::SphereParams::default(),
        );
        let result = mean_curvature_magnitude(&mesh);
        assert!(result.point_data().get_array("MeanCurvatureMag").is_some());
    }
    #[test]
    fn to_rgb() {
        let mut mesh = crate::filters::core::sources::sphere::sphere(
            &crate::filters::core::sources::sphere::SphereParams::default(),
        );
        mesh = gaussian_curvature(&mesh);
        let result = curvature_to_rgb(&mesh, "GaussianCurvature");
        assert!(result.point_data().get_array("CurvatureRGB").is_some());
    }
}
