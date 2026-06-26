use std::f64::consts::PI;

use crate::data::{CellArray, DataArray, Points, PolyData};

/// Parameters for generating a cylinder aligned along the Y axis.
pub struct CylinderParams {
    pub center: [f64; 3],
    pub height: f64,
    pub radius: f64,
    pub resolution: usize,
    pub capping: bool,
}

impl Default for CylinderParams {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0, 0.0],
            height: 1.0,
            radius: 0.5,
            resolution: 16,
            capping: true,
        }
    }
}

/// Generate a cylinder as PolyData, aligned along the Y axis.
pub fn cylinder(params: &CylinderParams) -> PolyData {
    let resolution = params.resolution.max(3);
    let angle = 2.0 * PI / resolution as f64;

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut tcoords = DataArray::<f64>::new("TCoords", 2);
    let mut polys = CellArray::new();

    for i in 0..resolution {
        let theta = i as f64 * angle;
        let nx = theta.cos();
        let nz = -theta.sin();
        let x = params.radius * nx + params.center[0];
        let z = params.radius * nz + params.center[2];
        let tcoord_x = (2.0 * i as f64 / resolution as f64 - 1.0).abs();

        points.push([x, 0.5 * params.height + params.center[1], z]);
        tcoords.push_tuple(&[tcoord_x, 0.0]);
        normals.push_tuple(&[nx, 0.0, nz]);

        points.push([x, -0.5 * params.height + params.center[1], z]);
        tcoords.push_tuple(&[tcoord_x, 1.0]);
        normals.push_tuple(&[nx, 0.0, nz]);
    }

    for i in 0..resolution {
        let pts0 = 2 * i;
        let pts1 = pts0 + 1;
        let pts2 = (pts1 + 2) % (2 * resolution);
        let pts3 = pts2 - 1;
        polys.push_cell(&[pts0 as i64, pts1 as i64, pts2 as i64, pts3 as i64]);
    }

    if params.capping {
        for i in 0..resolution {
            let theta = i as f64 * angle;
            let x = params.radius * theta.cos();
            let z = -params.radius * theta.sin();

            points.push([
                x + params.center[0],
                0.5 * params.height + params.center[1],
                z + params.center[2],
            ]);
            tcoords.push_tuple(&[x, z]);
            normals.push_tuple(&[0.0, 1.0, 0.0]);
        }
        for i in (0..resolution).rev() {
            let theta = i as f64 * angle;
            let x = params.radius * theta.cos();
            let z = -params.radius * theta.sin();

            points.push([
                x + params.center[0],
                -0.5 * params.height + params.center[1],
                z + params.center[2],
            ]);
            tcoords.push_tuple(&[x, z]);
            normals.push_tuple(&[0.0, -1.0, 0.0]);
        }

        let bottom: Vec<i64> = (0..resolution)
            .map(|i| (2 * resolution + i) as i64)
            .collect();
        polys.push_cell(&bottom);

        let top: Vec<i64> = (0..resolution)
            .map(|i| (3 * resolution + i) as i64)
            .collect();
        polys.push_cell(&top);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(normals.into());
    pd.point_data_mut().set_active_normals("Normals");
    pd.point_data_mut().add_array(tcoords.into());
    pd.point_data_mut().set_active_tcoords("TCoords");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cylinder() {
        let pd = cylinder(&CylinderParams::default());
        assert!(pd.points.len() > 0);
        assert_eq!(pd.points.len(), 64);
        assert_eq!(pd.polys.num_cells(), 16 + 2);
        assert_eq!(pd.polys.cell(16).len(), 16);
        assert_eq!(pd.polys.cell(17).len(), 16);
        assert!(pd.point_data().normals().is_some());
        assert!(pd.point_data().tcoords().is_some());
    }

    #[test]
    fn cylinder_no_cap() {
        let pd = cylinder(&CylinderParams {
            capping: false,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 32); // 16 bottom + 16 top ring
        assert_eq!(pd.polys.num_cells(), 16); // side quads only
        assert_eq!(pd.polys.cell(0), &[0, 1, 3, 2]);
    }
}
