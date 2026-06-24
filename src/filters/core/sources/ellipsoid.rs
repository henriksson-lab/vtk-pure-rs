use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::f64::consts::PI;

/// Parameters for generating an ellipsoid.
pub struct EllipsoidParams {
    /// Semi-axis length in X. Default: 1.0
    pub x_radius: f64,
    /// Semi-axis length in Y. Default: 0.5
    pub y_radius: f64,
    /// Semi-axis length in Z. Default: 0.25
    pub z_radius: f64,
    /// Number of meridian lines (longitude). Default: 32
    pub theta_resolution: usize,
    /// Number of latitude rings. Default: 16
    pub phi_resolution: usize,
    /// Center of the ellipsoid. Default: [0, 0, 0]
    pub center: [f64; 3],
}

impl Default for EllipsoidParams {
    fn default() -> Self {
        Self {
            x_radius: 1.0,
            y_radius: 0.5,
            z_radius: 0.25,
            theta_resolution: 32,
            phi_resolution: 16,
            center: [0.0, 0.0, 0.0],
        }
    }
}

/// Generate an ellipsoid as PolyData with smooth normals.
///
/// The ellipsoid is a UV sphere scaled by the three semi-axis radii.
pub fn ellipsoid(params: &EllipsoidParams) -> PolyData {
    let n_theta = params.theta_resolution.max(3);
    let n_phi = params.phi_resolution.max(2);
    let rx = params.x_radius;
    let ry = params.y_radius;
    let rz = params.z_radius;
    let cx = params.center[0];
    let cy = params.center[1];
    let cz = params.center[2];

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    // Top pole
    points.push([cx, cy, cz + rz]);
    normals.push_tuple(&[0.0, 0.0, 1.0]);

    // Intermediate rings
    for j in 1..n_phi {
        let phi = PI * j as f64 / n_phi as f64;
        let sp = phi.sin();
        let cp = phi.cos();

        for i in 0..n_theta {
            let theta = 2.0 * PI * i as f64 / n_theta as f64;
            let ct = theta.cos();
            let st = theta.sin();

            let x = cx + rx * sp * ct;
            let y = cy + ry * sp * st;
            let z = cz + rz * cp;
            points.push([x, y, z]);

            // Normal on ellipsoid: gradient of (x/rx)^2 + (y/ry)^2 + (z/rz)^2
            let nx = (x - cx) / (rx * rx);
            let ny = (y - cy) / (ry * ry);
            let nz = (z - cz) / (rz * rz);
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if len > 1e-15 {
                normals.push_tuple(&[nx / len, ny / len, nz / len]);
            } else {
                normals.push_tuple(&[0.0, 0.0, 1.0]);
            }
        }
    }

    // Bottom pole
    points.push([cx, cy, cz - rz]);
    normals.push_tuple(&[0.0, 0.0, -1.0]);

    let bottom_pole = points.len() as i64 - 1;

    // Top cap triangles
    for i in 0..n_theta {
        let i_next = (i + 1) % n_theta;
        polys.push_cell(&[0, (1 + i) as i64, (1 + i_next) as i64]);
    }

    // Body quads
    for j in 0..n_phi - 2 {
        for i in 0..n_theta {
            let i_next = (i + 1) % n_theta;
            let a = (1 + j * n_theta + i) as i64;
            let b = (1 + j * n_theta + i_next) as i64;
            let c = (1 + (j + 1) * n_theta + i_next) as i64;
            let d = (1 + (j + 1) * n_theta + i) as i64;
            polys.push_cell(&[a, b, c, d]);
        }
    }

    // Bottom cap triangles
    let last_ring_start = 1 + (n_phi - 2) * n_theta;
    for i in 0..n_theta {
        let i_next = (i + 1) % n_theta;
        polys.push_cell(&[
            (last_ring_start + i) as i64,
            bottom_pole,
            (last_ring_start + i_next) as i64,
        ]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(AnyDataArray::F64(normals));
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ellipsoid() {
        let pd = ellipsoid(&EllipsoidParams::default());
        // 2 poles + 15 rings * 32 pts = 482
        assert_eq!(pd.points.len(), 2 + 15 * 32);
        assert!(pd.polys.num_cells() > 0);
    }

    #[test]
    fn sphere_from_ellipsoid() {
        let pd = ellipsoid(&EllipsoidParams {
            x_radius: 1.0,
            y_radius: 1.0,
            z_radius: 1.0,
            theta_resolution: 8,
            phi_resolution: 4,
            ..Default::default()
        });
        // All points should be at distance 1 from center
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!((r - 1.0).abs() < 1e-10, "r={} at point {}", r, i);
        }
    }

    #[test]
    fn has_normals() {
        let pd = ellipsoid(&EllipsoidParams::default());
        assert!(pd.point_data().get_array("Normals").is_some());
    }

    #[test]
    fn custom_center() {
        let pd = ellipsoid(&EllipsoidParams {
            center: [5.0, 10.0, 15.0],
            theta_resolution: 4,
            phi_resolution: 2,
            ..Default::default()
        });
        // Top pole should be at [5, 10, 15.25]
        let p = pd.points.get(0);
        assert!((p[0] - 5.0).abs() < 1e-10);
        assert!((p[1] - 10.0).abs() < 1e-10);
    }
}
