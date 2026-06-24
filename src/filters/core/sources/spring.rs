use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::f64::consts::PI;

/// Parameters for generating a spring (coil/tube helix).
pub struct SpringParams {
    /// Radius of the helix path. Default: 1.0
    pub coil_radius: f64,
    /// Radius of the tube cross-section. Default: 0.1
    pub tube_radius: f64,
    /// Number of coils. Default: 5.0
    pub coils: f64,
    /// Height of the spring. Default: 2.0
    pub height: f64,
    /// Number of segments per coil around the helix. Default: 32
    pub coil_resolution: usize,
    /// Number of segments around the tube cross-section. Default: 8
    pub tube_resolution: usize,
    /// Center of the spring base. Default: [0, 0, 0]
    pub center: [f64; 3],
}

impl Default for SpringParams {
    fn default() -> Self {
        Self {
            coil_radius: 1.0,
            tube_radius: 0.1,
            coils: 5.0,
            height: 2.0,
            coil_resolution: 32,
            tube_resolution: 8,
            center: [0.0, 0.0, 0.0],
        }
    }
}

/// Generate a spring (helical tube) as PolyData.
pub fn spring(params: &SpringParams) -> PolyData {
    let n_helix = (params.coils * params.coil_resolution as f64).ceil() as usize + 1;
    let n_helix = n_helix.max(4);
    let n_tube = params.tube_resolution.max(3);
    let r_coil = params.coil_radius;
    let r_tube = params.tube_radius;

    let total_angle = 2.0 * PI * params.coils;

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    for i in 0..n_helix {
        let t = i as f64 / (n_helix - 1) as f64;
        let angle = total_angle * t;
        let z = params.center[2] + params.height * t;

        // Helix center at this position
        let hx = params.center[0] + r_coil * angle.cos();
        let hy = params.center[1] + r_coil * angle.sin();

        // Local frame: tangent, normal, binormal
        let tx = -r_coil * angle.sin() * total_angle;
        let ty = r_coil * angle.cos() * total_angle;
        let tz = params.height;
        let tlen = (tx * tx + ty * ty + tz * tz).sqrt();
        let tx = tx / tlen;
        let ty = ty / tlen;
        let tz = tz / tlen;

        // Normal = radial direction from helix axis
        let nx = angle.cos();
        let ny = angle.sin();
        let nz = 0.0;

        // Binormal = tangent × normal
        let bx = ty * nz - tz * ny;
        let by = tz * nx - tx * nz;
        let bz = tx * ny - ty * nx;

        // Generate tube cross-section
        for j in 0..n_tube {
            let phi = 2.0 * PI * j as f64 / n_tube as f64;
            let cp = phi.cos();
            let sp = phi.sin();

            let dx = r_tube * (cp * nx + sp * bx);
            let dy = r_tube * (cp * ny + sp * by);
            let dz = r_tube * (cp * nz + sp * bz);

            points.push([hx + dx, hy + dy, z + dz]);

            // Normal is the radial direction of the tube
            let nlen = (dx * dx + dy * dy + dz * dz).sqrt();
            if nlen > 1e-15 {
                normals.push_tuple(&[dx / nlen, dy / nlen, dz / nlen]);
            } else {
                normals.push_tuple(&[0.0, 0.0, 1.0]);
            }
        }
    }

    // Connect adjacent rings with quads
    for i in 0..n_helix - 1 {
        for j in 0..n_tube {
            let j_next = (j + 1) % n_tube;
            let a = (i * n_tube + j) as i64;
            let b = (i * n_tube + j_next) as i64;
            let c = ((i + 1) * n_tube + j_next) as i64;
            let d = ((i + 1) * n_tube + j) as i64;
            polys.push_cell(&[a, b, c, d]);
        }
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
    fn default_spring() {
        let pd = spring(&SpringParams::default());
        assert!(pd.points.len() > 100);
        assert!(pd.polys.num_cells() > 100);
    }

    #[test]
    fn small_spring() {
        let pd = spring(&SpringParams {
            coils: 1.0,
            coil_resolution: 4,
            tube_resolution: 3,
            ..Default::default()
        });
        // 5 helix stations * 3 tube points = 15 points
        assert_eq!(pd.points.len(), 15);
        // 4 helix segments * 3 tube segments = 12 quads
        assert_eq!(pd.polys.num_cells(), 12);
    }

    #[test]
    fn has_normals() {
        let pd = spring(&SpringParams::default());
        assert!(pd.point_data().get_array("Normals").is_some());
    }
}
