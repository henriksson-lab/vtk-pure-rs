use crate::data::{CellArray, DataArray, Points, PolyData};
use crate::filters::core::sources::mobius::evaluate_mobius;

/// Parameters for generating a Möbius strip surface with normals.
pub struct MobiusStripParams {
    /// Radius of the center circle. Default: 1.0
    pub radius: f64,
    /// Half-width of the strip. Default: 0.3
    pub width: f64,
    /// Number of segments around the loop. Default: 64
    pub resolution: usize,
    /// Number of subdivisions across the strip width. Default: 4
    pub width_resolution: usize,
    /// Center of the strip. Default: [0, 0, 0]
    pub center: [f64; 3],
}

impl Default for MobiusStripParams {
    fn default() -> Self {
        Self {
            radius: 1.0,
            width: 0.3,
            resolution: 64,
            width_resolution: 4,
            center: [0.0, 0.0, 0.0],
        }
    }
}

/// Generate a Möbius strip as a triangulated PolyData surface with normals.
pub fn mobius_strip(params: &MobiusStripParams) -> PolyData {
    let pts_u = params.resolution.max(8);
    let pts_v = params.width_resolution.max(1) + 1;
    let radius = params.radius;
    let half_width = params.width;
    let [cx, cy, cz] = params.center;

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    for i in 0..pts_u {
        let u = 2.0 * std::f64::consts::PI * i as f64 / (pts_u - 1) as f64;
        for j in 0..pts_v {
            let v = -half_width + 2.0 * half_width * j as f64 / (pts_v - 1) as f64;
            let (pt, du, dv) = evaluate_mobius(u, v, radius);
            points.push([cx + pt[0], cy + pt[1], cz + pt[2]]);

            let nx = du[1] * dv[2] - du[2] * dv[1];
            let ny = du[2] * dv[0] - du[0] * dv[2];
            let nz = du[0] * dv[1] - du[1] * dv[0];
            let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-12);
            normals.push_tuple(&[nx / len, ny / len, nz / len]);
        }
    }

    for i in 0..(pts_u - 1) {
        for j in 0..(pts_v - 1) {
            let id1 = (j + i * pts_v) as i64;
            let id2 = id1 + pts_v as i64;
            let id3 = id2 + 1;
            let id4 = id1 + 1;
            polys.push_cell(&[id1, id2, id3]);
            polys.push_cell(&[id1, id3, id4]);
        }
    }
    for j in 0..(pts_v - 1) {
        let id1 = (j + (pts_u - 1) * pts_v) as i64;
        let id3 = id1 + 1;
        let id2 = (pts_v - 1 - j) as i64;
        let id4 = id2 - 1;
        polys.push_cell(&[id1, id2, id3]);
        polys.push_cell(&[id1, id3, id4]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(normals.into());
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mobius_strip() {
        let pd = mobius_strip(&MobiusStripParams::default());
        // 64 * (4+1) = 320 points
        assert_eq!(pd.points.len(), 320);
        // 64 * 4 * 2 = 512 triangles
        assert_eq!(pd.polys.num_cells(), 512);
        assert!(pd.point_data().normals().is_some());
    }

    #[test]
    fn minimal_mobius_strip() {
        let pd = mobius_strip(&MobiusStripParams {
            resolution: 8,
            width_resolution: 1,
            ..Default::default()
        });
        // 8 * (1+1) = 16 points
        assert_eq!(pd.points.len(), 16);
        // 8 * 1 * 2 = 16 triangles
        assert_eq!(pd.polys.num_cells(), 16);
    }

    #[test]
    fn custom_center() {
        let pd = mobius_strip(&MobiusStripParams {
            center: [1.0, 2.0, 3.0],
            resolution: 8,
            width_resolution: 1,
            ..Default::default()
        });
        assert!(pd.points.len() > 0);
    }
}
