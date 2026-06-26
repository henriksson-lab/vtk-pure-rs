//! UV mapping generators for meshes.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Generate planar UV projection along a given axis.
pub fn uv_planar(mesh: &PolyData, axis: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        let mut result = mesh.clone();
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("UV", Vec::new(), 2)));
        result.point_data_mut().set_active_tcoords("UV");
        return result;
    }
    let (u_axis, v_axis) = match axis {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let u_min = pts.iter().map(|p| p[u_axis]).fold(f64::INFINITY, f64::min);
    let u_max = pts
        .iter()
        .map(|p| p[u_axis])
        .fold(f64::NEG_INFINITY, f64::max);
    let v_min = pts.iter().map(|p| p[v_axis]).fold(f64::INFINITY, f64::min);
    let v_max = pts
        .iter()
        .map(|p| p[v_axis])
        .fold(f64::NEG_INFINITY, f64::max);
    let u_range = if (u_max - u_min).abs() < 1e-15 {
        1.0
    } else {
        u_max - u_min
    };
    let v_range = if (v_max - v_min).abs() < 1e-15 {
        1.0
    } else {
        v_max - v_min
    };

    let data: Vec<f64> = pts
        .iter()
        .flat_map(|p| vec![(p[u_axis] - u_min) / u_range, (p[v_axis] - v_min) / v_range])
        .collect();

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("UV", data, 2)));
    result.point_data_mut().set_active_tcoords("UV");
    result
}

/// Generate cylindrical UV mapping around Z axis.
pub fn uv_cylindrical(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        let mut result = mesh.clone();
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("UV", Vec::new(), 2)));
        result.point_data_mut().set_active_tcoords("UV");
        return result;
    }
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let z_min = pts.iter().map(|p| p[2]).fold(f64::INFINITY, f64::min);
    let z_max = pts.iter().map(|p| p[2]).fold(f64::NEG_INFINITY, f64::max);
    let z_range = if (z_max - z_min).abs() < 1e-15 {
        1.0
    } else {
        z_max - z_min
    };

    let data: Vec<f64> = pts
        .iter()
        .flat_map(|p| {
            let radius = (p[0] * p[0] + p[1] * p[1]).sqrt();
            let u = if radius > 1e-15 {
                (p[0] / radius).clamp(-1.0, 1.0).acos() / std::f64::consts::PI
            } else {
                0.0
            };
            let v = (p[2] - z_min) / z_range;
            vec![u, v]
        })
        .collect();

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("UV", data, 2)));
    result.point_data_mut().set_active_tcoords("UV");
    result
}

/// Generate spherical UV mapping.
pub fn uv_spherical(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        let mut result = mesh.clone();
        result
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("UV", Vec::new(), 2)));
        result.point_data_mut().set_active_tcoords("UV");
        return result;
    }

    let mut center = [0.0; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        center[0] += p[0];
        center[1] += p[1];
        center[2] += p[2];
    }
    center[0] /= n as f64;
    center[1] /= n as f64;
    center[2] /= n as f64;

    let data: Vec<f64> = (0..n)
        .flat_map(|i| {
            let p = mesh.points.get(i);
            let x = p[0] - center[0];
            let y = p[1] - center[1];
            let z = p[2] - center[2];
            let rho = (x * x + y * y + z * z).sqrt();
            let phi = if rho > 1e-15 {
                (z / rho).clamp(-1.0, 1.0).acos()
            } else {
                0.0
            };
            let v = phi / std::f64::consts::PI;
            let r = rho * phi.sin();
            let u = if r > 1e-15 {
                (x / r).clamp(-1.0, 1.0).acos() / std::f64::consts::PI
            } else {
                0.0
            };
            vec![u, v]
        })
        .collect();

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("UV", data, 2)));
    result.point_data_mut().set_active_tcoords("UV");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_planar() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = uv_planar(&mesh, 2); // project along Z
        let arr = r.point_data().get_array("UV").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0; 2];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10); // x=0 -> u=0
    }
    #[test]
    fn test_cylindrical() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.5], [-1.0, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let r = uv_cylindrical(&mesh);
        assert!(r.point_data().get_array("UV").is_some());
    }
    #[test]
    fn test_spherical() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let r = uv_spherical(&mesh);
        let arr = r.point_data().get_array("UV").unwrap();
        assert_eq!(arr.num_components(), 2);
    }
}
