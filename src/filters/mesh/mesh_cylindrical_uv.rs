//! Cylindrical UV mapping for mesh unwrapping.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn cylindrical_uv(mesh: &PolyData, axis: [f64; 3]) -> PolyData {
    cylindrical_uv_with_prevent_seam(mesh, axis, true)
}

pub fn cylindrical_uv_with_prevent_seam(
    mesh: &PolyData,
    axis: [f64; 3],
    prevent_seam: bool,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut ax = axis;
    if normalize(&mut ax) == 0.0 {
        ax = [0.0, 0.0, 1.0];
    }

    let mut z_min = f64::INFINITY;
    let mut z_max = f64::NEG_INFINITY;
    for i in 0..n {
        let p = mesh.points.get(i);
        let z = dot(p, ax);
        z_min = z_min.min(z);
        z_max = z_max.max(z);
    }

    let (z_min, z_max) = if z_min == z_max {
        (z_min - 0.5, z_max + 0.5)
    } else {
        (z_min, z_max)
    };
    let point1 = [ax[0] * z_min, ax[1] * z_min, ax[2] * z_min];
    let point2 = [ax[0] * z_max, ax[1] * z_max, ax[2] * z_max];
    let axis_vec = [
        point2[0] - point1[0],
        point2[1] - point1[1],
        point2[2] - point1[2],
    ];
    let axis_len2 = dot(axis_vec, axis_vec);
    if axis_len2 == 0.0 {
        return mesh.clone();
    }

    let mut v = [1.0, 0.0, 0.0];
    let mut vp = cross(axis_vec, v);
    if norm(vp) == 0.0 {
        v = [0.0, 1.0, 0.0];
        vp = cross(axis_vec, v);
    }
    let mut vec = cross(vp, axis_vec);
    if normalize(&mut vec) == 0.0 {
        return mesh.clone();
    }

    let mut uvs = Vec::with_capacity(n * 2);
    for i in 0..n {
        let p = mesh.points.get(i);
        let t = dot(
            [p[0] - point1[0], p[1] - point1[1], p[2] - point1[2]],
            axis_vec,
        ) / axis_len2;
        let closest_t = t.clamp(0.0, 1.0);
        let closest = [
            point1[0] + closest_t * axis_vec[0],
            point1[1] + closest_t * axis_vec[1],
            point1[2] + closest_t * axis_vec[2],
        ];
        let mut radial = [p[0] - closest[0], p[1] - closest[1], p[2] - closest[2]];
        normalize(&mut radial);

        let theta_x = dot(radial, vec).clamp(-1.0, 1.0).acos();
        let vp = cross(vec, radial);
        let theta_y = dot(axis_vec, vp);
        let u = if prevent_seam {
            theta_x / std::f64::consts::PI
        } else {
            let mut u = theta_x / (2.0 * std::f64::consts::PI);
            if theta_y < 0.0 {
                u = 1.0 - u;
            }
            u
        };
        let v = t;
        uvs.push(u);
        uvs.push(v);
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("TCoords", uvs, 2)));
    result.point_data_mut().set_active_tcoords("TCoords");
    result
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

fn normalize(a: &mut [f64; 3]) -> f64 {
    let len = norm(*a);
    if len != 0.0 {
        a[0] /= len;
        a[1] /= len;
        a[2] /= len;
    }
    len
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
        assert!(r.point_data().tcoords().is_some());
    }

    #[test]
    fn test_cyl_uv_without_prevent_seam_uses_full_angle() {
        let mesh = PolyData::from_points(vec![[0.0, -1.0, 0.0], [0.0, 1.0, 0.0]]);
        let r = cylindrical_uv_with_prevent_seam(&mesh, [0.0, 0.0, 1.0], false);
        let arr = r.point_data().tcoords().unwrap();
        let mut u0 = [0.0, 0.0];
        let mut u1 = [0.0, 0.0];
        arr.tuple_as_f64(0, &mut u0);
        arr.tuple_as_f64(1, &mut u1);
        assert!((u0[0] - 0.75).abs() < 1e-10);
        assert!((u1[0] - 0.25).abs() < 1e-10);
    }
}
