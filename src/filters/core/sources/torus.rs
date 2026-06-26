use crate::data::{CellArray, DataArray, Points, PolyData};
use std::f64::consts::PI;

/// Parameters for generating a torus.
pub struct TorusParams {
    /// Major radius (center of tube to center of torus). Default: 1.0
    pub ring_radius: f64,
    /// Minor radius (radius of the tube). Default: 0.5
    pub cross_section_radius: f64,
    /// Number of segments around the ring. Default: 32
    pub ring_resolution: usize,
    /// Number of segments around the cross-section. Default: 16
    pub cross_section_resolution: usize,
    /// Center of the torus. Default: [0, 0, 0]
    pub center: [f64; 3],
}

impl Default for TorusParams {
    fn default() -> Self {
        Self {
            ring_radius: 1.0,
            cross_section_radius: 0.5,
            ring_resolution: 32,
            cross_section_resolution: 16,
            center: [0.0, 0.0, 0.0],
        }
    }
}

/// Generate a torus in the XY plane as PolyData with smooth normals.
pub fn torus(params: &TorusParams) -> PolyData {
    let n_ring = params.ring_resolution.max(3);
    let n_cs = params.cross_section_resolution.max(3);
    let r_ring = params.ring_radius;
    let r_cs = params.cross_section_radius;
    let cx = params.center[0];
    let cy = params.center[1];
    let cz = params.center[2];

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    // Generate vertices and normals
    for i in 0..n_ring {
        let theta = 2.0 * PI * i as f64 / n_ring as f64;
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        for j in 0..n_cs {
            let phi = 2.0 * PI * j as f64 / n_cs as f64;
            let cos_p = phi.cos();
            let sin_p = phi.sin();

            let r = r_ring + r_cs * cos_p;
            let x = cx + r * sin_t;
            let y = cy + r * cos_t;
            let z = cz + r_cs * sin_p;

            points.push([x, y, z]);

            // Normal points outward from the tube center
            let nx = cos_p * sin_t;
            let ny = cos_p * cos_t;
            let nz = sin_p;
            normals.push_tuple(&[nx, ny, nz]);
        }
    }

    // Generate quad faces
    for i in 0..n_ring {
        let i_next = (i + 1) % n_ring;
        for j in 0..n_cs {
            let j_next = (j + 1) % n_cs;

            let a = (i * n_cs + j) as i64;
            let b = (i * n_cs + j_next) as i64;
            let c = (i_next * n_cs + j_next) as i64;
            let d = (i_next * n_cs + j) as i64;

            polys.push_cell(&[a, b, c, d]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut()
        .add_array(crate::data::AnyDataArray::F64(normals));
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_torus() {
        let pd = torus(&TorusParams::default());
        assert_eq!(pd.points.len(), 32 * 16);
        assert_eq!(pd.polys.num_cells(), 32 * 16);
    }

    #[test]
    fn small_torus() {
        let pd = torus(&TorusParams {
            ring_resolution: 4,
            cross_section_resolution: 3,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 12);
        assert_eq!(pd.polys.num_cells(), 12);
    }

    #[test]
    fn has_normals() {
        let pd = torus(&TorusParams::default());
        assert!(pd.point_data().get_array("Normals").is_some());
    }

    #[test]
    fn custom_center() {
        let pd = torus(&TorusParams {
            center: [10.0, 20.0, 30.0],
            ring_resolution: 4,
            cross_section_resolution: 3,
            ..Default::default()
        });
        // All points should be near the center
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            assert!((p[0] - 10.0).abs() < 2.0);
            assert!((p[1] - 20.0).abs() < 2.0);
            assert!((p[2] - 30.0).abs() < 1.0);
        }
    }
}
