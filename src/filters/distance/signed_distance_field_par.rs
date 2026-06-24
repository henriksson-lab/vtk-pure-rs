//! Signed distance field from oriented point clouds (parallel).
//!
//! Computes a signed distance field on an ImageData grid from an oriented
//! point cloud using rayon for parallelization.

use crate::data::{AnyDataArray, DataArray, ImageData, PolyData};
use rayon::prelude::*;

/// Compute a signed distance field from a point cloud with normals.
///
/// For each grid point, finds the closest point in the cloud and
/// uses the dot product with the normal to determine the sign.
/// Positive = outside, negative = inside.
pub fn signed_distance_field_from_points(
    points: &PolyData,
    dims: [usize; 3],
    spacing: [f64; 3],
    origin: [f64; 3],
) -> ImageData {
    let n_pts = points.points.len();
    if n_pts == 0 {
        return ImageData::with_dimensions(dims[0], dims[1], dims[2])
            .with_spacing(spacing)
            .with_origin(origin);
    }

    let normals = points.point_data().normals();
    let has_normals = normals.is_some() && normals.unwrap().num_components() == 3;

    // Precompute point positions
    let src_pts: Vec<[f64; 3]> = (0..n_pts).map(|i| points.points.get(i)).collect();
    let src_normals: Vec<[f64; 3]> = if has_normals {
        let n_arr = normals.unwrap();
        let mut buf = [0.0f64; 3];
        (0..n_pts)
            .map(|i| {
                n_arr.tuple_as_f64(i, &mut buf);
                buf
            })
            .collect()
    } else {
        vec![[0.0, 0.0, 1.0]; n_pts]
    };

    let total = dims[0] * dims[1] * dims[2];

    // Parallel SDF computation
    let sdf: Vec<f64> = (0..total)
        .into_par_iter()
        .map(|idx| {
            let iz = idx / (dims[0] * dims[1]);
            let rem = idx % (dims[0] * dims[1]);
            let iy = rem / dims[0];
            let ix = rem % dims[0];

            let gx = origin[0] + ix as f64 * spacing[0];
            let gy = origin[1] + iy as f64 * spacing[1];
            let gz = origin[2] + iz as f64 * spacing[2];

            // Find closest point
            let mut best_dist2 = f64::MAX;
            let mut best_idx = 0;
            for (pi, sp) in src_pts.iter().enumerate() {
                let dx = gx - sp[0];
                let dy = gy - sp[1];
                let dz = gz - sp[2];
                let d2 = dx * dx + dy * dy + dz * dz;
                if d2 < best_dist2 {
                    best_dist2 = d2;
                    best_idx = pi;
                }
            }

            let dist = best_dist2.sqrt();

            // Sign from normal
            let sp = &src_pts[best_idx];
            let sn = &src_normals[best_idx];
            let dx = gx - sp[0];
            let dy = gy - sp[1];
            let dz = gz - sp[2];
            let dot = dx * sn[0] + dy * sn[1] + dz * sn[2];

            if dot >= 0.0 {
                dist
            } else {
                -dist
            }
        })
        .collect();

    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(spacing)
        .with_origin(origin)
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "SignedDistance",
            sdf,
            1,
        )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Points;

    #[test]
    fn sphere_sdf() {
        // Points on unit sphere with outward normals
        let mut mesh = PolyData::new();
        let mut pts = Vec::new();
        let mut normals = Vec::new();
        let n = 50;
        for i in 0..n {
            let theta = std::f64::consts::PI * i as f64 / n as f64;
            for j in 0..n {
                let phi = 2.0 * std::f64::consts::PI * j as f64 / n as f64;
                let x = theta.sin() * phi.cos();
                let y = theta.sin() * phi.sin();
                let z = theta.cos();
                pts.push([x, y, z]);
                normals.push(x);
                normals.push(y);
                normals.push(z);
            }
        }
        mesh.points = Points::from(pts);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals", normals, 3,
            )));
        mesh.point_data_mut().set_active_normals("Normals");

        let sdf = signed_distance_field_from_points(
            &mesh,
            [10, 10, 10],
            [0.3, 0.3, 0.3],
            [-1.5, -1.5, -1.5],
        );
        let arr = sdf.point_data().get_array("SignedDistance").unwrap();

        // Center should be negative (inside)
        let center_idx = 5 + 5 * 10 + 5 * 100;
        let mut buf = [0.0f64];
        arr.tuple_as_f64(center_idx, &mut buf);
        assert!(
            buf[0] < 0.0,
            "center SDF should be negative, got {}",
            buf[0]
        );

        // Corner should be positive (outside)
        arr.tuple_as_f64(0, &mut buf);
        assert!(
            buf[0] > 0.0,
            "corner SDF should be positive, got {}",
            buf[0]
        );
    }

    #[test]
    fn empty_points() {
        let mesh = PolyData::new();
        let sdf =
            signed_distance_field_from_points(&mesh, [5, 5, 5], [1.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
        assert_eq!(sdf.dimensions(), [5, 5, 5]);
    }
}
