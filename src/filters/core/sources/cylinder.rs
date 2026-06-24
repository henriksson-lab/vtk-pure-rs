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
    let n = params.resolution.max(3);
    let [cx, cy, cz] = params.center;
    let h = params.height;
    let r = params.radius;
    let hy = h * 0.5;

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    // Bottom ring (points 0..n)
    for i in 0..n {
        let theta = 2.0 * PI * i as f64 / n as f64;
        let ct = theta.cos();
        let st = theta.sin();
        points.push([cx + r * ct, cy - hy, cz + r * st]);
        normals.push_tuple(&[ct, 0.0, st]);
    }

    // Top ring (points n..2n)
    for i in 0..n {
        let theta = 2.0 * PI * i as f64 / n as f64;
        let ct = theta.cos();
        let st = theta.sin();
        points.push([cx + r * ct, cy + hy, cz + r * st]);
        normals.push_tuple(&[ct, 0.0, st]);
    }

    // Side quads
    for i in 0..n {
        let next = (i + 1) % n;
        polys.push_cell(&[i as i64, next as i64, (n + next) as i64, (n + i) as i64]);
    }

    // Caps
    if params.capping {
        // Bottom cap
        let base = points.len() as i64;
        points.push([cx, cy - hy, cz]);
        normals.push_tuple(&[0.0, -1.0, 0.0]);
        for i in 0..n {
            let theta = 2.0 * PI * i as f64 / n as f64;
            points.push([cx + r * theta.cos(), cy - hy, cz + r * theta.sin()]);
            normals.push_tuple(&[0.0, -1.0, 0.0]);
        }
        for i in 0..n {
            let next = (i + 1) % n;
            polys.push_cell(&[base, base + 1 + next as i64, base + 1 + i as i64]);
        }

        // Top cap
        let base = points.len() as i64;
        points.push([cx, cy + hy, cz]);
        normals.push_tuple(&[0.0, 1.0, 0.0]);
        for i in 0..n {
            let theta = 2.0 * PI * i as f64 / n as f64;
            points.push([cx + r * theta.cos(), cy + hy, cz + r * theta.sin()]);
            normals.push_tuple(&[0.0, 1.0, 0.0]);
        }
        for i in 0..n {
            let next = (i + 1) % n;
            polys.push_cell(&[base, base + 1 + i as i64, base + 1 + next as i64]);
        }
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
    fn default_cylinder() {
        let pd = cylinder(&CylinderParams::default());
        assert!(pd.points.len() > 0);
        // Side quads + top cap triangles + bottom cap triangles
        assert_eq!(pd.polys.num_cells(), 16 + 16 + 16);
    }

    #[test]
    fn cylinder_no_cap() {
        let pd = cylinder(&CylinderParams {
            capping: false,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 32); // 16 bottom + 16 top ring
        assert_eq!(pd.polys.num_cells(), 16); // side quads only
    }
}
