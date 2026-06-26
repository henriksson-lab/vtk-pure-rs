//! UV-mapped sphere with texture coordinates.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Create a vtkTexturedSphereSource-style sphere with texture coordinates.
pub fn sphere_uv(radius: f64, u_res: usize, v_res: usize) -> PolyData {
    let theta_resolution = u_res.max(4);
    let phi_resolution = v_res.max(4);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut normals = Vec::with_capacity((theta_resolution + 1) * (phi_resolution + 1) * 3);
    let mut tcoords = Vec::with_capacity((theta_resolution + 1) * (phi_resolution + 1) * 2);

    let delta_phi = std::f64::consts::PI / phi_resolution as f64;
    let delta_theta = 2.0 * std::f64::consts::PI / theta_resolution as f64;

    for i in 0..=theta_resolution {
        let theta = i as f64 * delta_theta;
        let tc0 = theta / (2.0 * std::f64::consts::PI);
        for j in 0..=phi_resolution {
            let phi = j as f64 * delta_phi;
            let ring_radius = radius * phi.sin();
            let x = ring_radius * theta.cos();
            let y = ring_radius * theta.sin();
            let z = radius * phi.cos();
            pts.push([x, y, z]);

            let mut norm = (x * x + y * y + z * z).sqrt();
            if norm == 0.0 {
                norm = 1.0;
            }
            normals.extend_from_slice(&[x / norm, y / norm, z / norm]);
            tcoords.extend_from_slice(&[tc0, 1.0 - phi / std::f64::consts::PI]);
        }
    }

    for i in 0..theta_resolution {
        for j in 0..phi_resolution {
            let p0 = ((phi_resolution + 1) * i + j) as i64;
            let p1 = p0 + 1;
            let p2 = ((phi_resolution + 1) * (i + 1) + j + 1) as i64;
            polys.push_cell(&[p0, p1, p2]);
            polys.push_cell(&[p0, p2, p2 - 1]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Normals", normals, 3,
        )));
    result.point_data_mut().set_active_normals("Normals");
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "TCoords", tcoords, 2,
        )));
    result.point_data_mut().set_active_tcoords("TCoords");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sphere_uv() {
        let s = sphere_uv(1.0, 16, 8);
        assert_eq!(s.points.len(), (16 + 1) * (8 + 1));
        assert_eq!(s.polys.num_cells(), 16 * 8 * 2);
        assert!(s.point_data().normals().is_some());
        assert!(s.point_data().get_array("TCoords").is_some());
        assert_eq!(
            s.point_data()
                .get_array("TCoords")
                .unwrap()
                .num_components(),
            2
        );
    }
}
