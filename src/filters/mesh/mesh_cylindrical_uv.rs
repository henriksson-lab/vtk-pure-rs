//! Cylindrical UV mapping for mesh unwrapping.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn cylindrical_uv(mesh: &PolyData, axis: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let al = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    let ax = if al > 1e-15 {
        [axis[0] / al, axis[1] / al, axis[2] / al]
    } else {
        [0.0, 0.0, 1.0]
    };
    // Find two orthogonal vectors to axis
    let up = if ax[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u_dir = [
        ax[1] * up[2] - ax[2] * up[1],
        ax[2] * up[0] - ax[0] * up[2],
        ax[0] * up[1] - ax[1] * up[0],
    ];
    let ul = (u_dir[0] * u_dir[0] + u_dir[1] * u_dir[1] + u_dir[2] * u_dir[2]).sqrt();
    let u_dir = [u_dir[0] / ul, u_dir[1] / ul, u_dir[2] / ul];
    let v_dir = [
        ax[1] * u_dir[2] - ax[2] * u_dir[1],
        ax[2] * u_dir[0] - ax[0] * u_dir[2],
        ax[0] * u_dir[1] - ax[1] * u_dir[0],
    ];
    let mut uvs = Vec::with_capacity(n * 2);
    let mut z_min = f64::INFINITY;
    let mut z_max = f64::NEG_INFINITY;
    let zvals: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            let z = p[0] * ax[0] + p[1] * ax[1] + p[2] * ax[2];
            if z < z_min {
                z_min = z;
            }
            if z > z_max {
                z_max = z;
            }
            z
        })
        .collect();
    for i in 0..n {
        let p = mesh.points.get(i);
        let pu = p[0] * u_dir[0] + p[1] * u_dir[1] + p[2] * u_dir[2];
        let pv = p[0] * v_dir[0] + p[1] * v_dir[1] + p[2] * v_dir[2];
        let angle = pv.atan2(pu);
        let u = (angle + std::f64::consts::PI) / (2.0 * std::f64::consts::PI);
        let v = if (z_max - z_min).abs() > 1e-15 {
            (zvals[i] - z_min) / (z_max - z_min)
        } else {
            0.5
        };
        uvs.push(u);
        uvs.push(v);
    }
    let mut result = mesh.clone();
    let u_vals: Vec<f64> = (0..n).map(|i| uvs[i * 2]).collect();
    let v_vals: Vec<f64> = (0..n).map(|i| uvs[i * 2 + 1]).collect();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("U", u_vals, 1)));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("V", v_vals, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cyl_uv() {
        let mesh = PolyData::from_triangles(
            vec![
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.5],
                [0.0, 0.0, 1.0],
                [-1.0, 0.0, 0.5],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let r = cylindrical_uv(&mesh, [0.0, 0.0, 1.0]);
        assert!(r.point_data().get_array("U").is_some());
        assert!(r.point_data().get_array("V").is_some());
    }
}
